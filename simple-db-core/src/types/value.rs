//! Tagged-pointer value type for representing any supported database column value in a single word.
//!
//! The high 16 bits of the word encode the type tag; the low 48 bits (on x86_64) hold either
//! the value itself (for types that fit inline) or a heap pointer. This avoids a `Box<dyn Any>`
//! allocation for every small scalar and keeps the type check to a single mask operation.

use std::{any::Any, ptr::NonNull};

use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use serde_json::Value as JsonValue;
use rust_decimal::Decimal;
use smol_str::SmolStr;
use uuid::Uuid;

#[cfg(target_arch = "x86_64")]
type Word = u64;

#[cfg(not(target_arch = "x86_64"))]
type Word = u128;

const TAG_BITS:        u32 = 16;
const TAG_SHIFT:       u32 = Word::BITS - TAG_BITS;
const PAYLOAD_BITS:    u32 = TAG_SHIFT;
const PAYLOAD_MASK:    Word = Word::MAX >> TAG_BITS;
const CATEGORY_MASK:   Word = 0b1100000000000000;
const TYPE_MASK:       Word = 0b0011111111111111;
const CATEGORY_INLINE: Word = 0b0000000000000000;
const CATEGORY_BOXED:  Word = 0b0100000000000000;

const TYPE_NULL:       Word = 0;
const TYPE_BOOL:       Word = 1;
const TYPE_I8:         Word = 2;
const TYPE_I16:        Word = 3;
const TYPE_I32:        Word = 4;
const TYPE_I64:        Word = 5;
const TYPE_I128:       Word = 6;
const TYPE_U8:         Word = 7;
const TYPE_U16:        Word = 8;
const TYPE_U32:        Word = 9;
const TYPE_U64:        Word = 10;
const TYPE_U128:       Word = 11;
const TYPE_F32:        Word = 12;
const TYPE_F64:        Word = 13;
const TYPE_DECIMAL:    Word = 14;
const TYPE_CHAR:       Word = 15;
const TYPE_STRING:     Word = 16;
const TYPE_DATE:       Word = 17;
const TYPE_TIME:       Word = 18;
const TYPE_TIMESTAMP:  Word = 19;
const TYPE_TIMESTAMPZ: Word = 20;
const TYPE_BYTES:      Word = 21;
const TYPE_UUID:       Word = 22;
const TYPE_JSON:       Word = 23;
const TYPE_EXTENSION:  Word = TYPE_MASK;

// ========================================
// 1. Serialization (Rust -> DB)
// ========================================

/// Converts a Rust value into a [`DbValue`].
///
/// Implement this to make a type usable as a query parameter or column value.
pub trait ToDbValue {
    fn to_db_value(&self) -> DbValue;
}

// ========================================
// 2. Deserialization (DB -> Rust)
// ========================================

/// Extracts a Rust value from a [`DbValue`].
///
/// Three levels of extraction are provided:
/// - [`matches_type`](FromDbValue::matches_type) — zero-cost tag check only
/// - [`from_exact`](FromDbValue::from_exact) — strict: fails if the stored type doesn't match exactly
/// - [`from_cast`](FromDbValue::from_cast) — loose: may parse strings, widen integers, etc.
pub trait FromDbValue: Sized {
    /// Returns `true` if the stored type tag matches `Self` exactly.
    fn matches_type(value: &DbValue) -> bool;

    /// Extracts `Self` without any coercion. Returns `None` on type mismatch.
    fn from_exact(value: &DbValue) -> Option<Self>;

    /// Extracts `Self` with best-effort coercion (string parsing, integer widening, etc.).
    fn from_cast(value: &DbValue) -> Option<Self>;
}

// ========================================
// 3. The Custom Extension Trait
// ========================================

/// Marker trait for user-defined types that can be stored as a [`DbValue`].
///
/// Implement this alongside [`ToDbValue`] / [`FromDbValue`] to add support for
/// application-specific column types (e.g. enums, newtype wrappers).
pub trait CustomDbValue: Send + Sync + 'static {
    /// Clones the value into a new `Box<dyn CustomDbValue>` so rows can be cloned in memory.
    fn clone_box(&self) -> Box<dyn CustomDbValue>;

    /// Returns a `&dyn Any` to enable runtime downcasting back to the concrete type.
    fn as_any(&self) -> &dyn Any;

    /// Dynamic equality check used by [`DbValue`]'s `PartialEq` implementation.
    fn dyn_eq(&self, other: &dyn CustomDbValue) -> bool;

    /// Dynamic ordering check used by [`DbValue`]'s `PartialOrd` implementation.
    fn dyn_partial_cmp(&self, other: &dyn CustomDbValue) -> Option<std::cmp::Ordering>;
}

#[inline]
fn make_tag(category: Word, r#type: Word) -> Word {
    debug_assert_eq!(category & !CATEGORY_MASK, 0);
    debug_assert_eq!(r#type & !TYPE_MASK, 0);
    category | r#type
}

/// A type-tagged database column value stored in a single machine word.
///
/// Small scalars (`bool`, `i8`–`i32`, small `i64`/`u64`, `f32`, `char`) are stored entirely
/// inline — no heap allocation. Larger types (`String`, `Vec<u8>`, temporal types, etc.) are
/// heap-allocated and the pointer is stored in the payload field.
///
/// # Type Safety
/// Every extraction via [`get`](DbValue::get) or [`cast`](DbValue::cast) is guarded by a tag
/// check; an incorrect type always returns `None` rather than causing undefined behaviour.
#[derive(Debug)]
pub struct DbValue(Word);

unsafe impl Send for DbValue {}
unsafe impl Sync for DbValue {}

impl DbValue {
    #[inline]
    fn from_tag_and_payload(tag: Word, payload: Word) -> Self {
        debug_assert!(tag < ((1 as Word) << TAG_BITS));
        debug_assert_eq!(payload & !PAYLOAD_MASK, 0);
        Self((tag << TAG_SHIFT) | payload)
    }

    #[inline]
    fn from_tag_and_boxed<T>(tag: Word, val: T) -> Self {
        let raw = Box::into_raw(Box::new(val));
        let ptr = NonNull::new(raw).expect("Box::into_raw returned null");
        let addr = ptr.as_ptr() as usize as Word;
        assert!(addr & !PAYLOAD_MASK == 0, "allocator returned address above payload range ({} bits); pointer tagging is unsafe on this platform", PAYLOAD_BITS);
        Self::from_tag_and_payload(tag, addr & PAYLOAD_MASK)
    }

    #[inline]
    fn tag(&self) -> Word {
        self.0 >> TAG_SHIFT
    }

    #[inline]
    fn category(&self) -> Word {
        self.tag() & CATEGORY_MASK
    }

    #[inline]
    fn r#type(&self) -> Word {
        self.tag() & TYPE_MASK
    }

    #[inline]
    fn payload(&self) -> Word {
        self.0 & PAYLOAD_MASK
    }

    /// Reinterprets the 48-bit payload as a sign-extended `i64`.
    /// Used for `i64` values stored inline when they fit in 48 bits.
    #[inline]
    fn payload_to_i64_i48(&self) -> i64 {
        let p = self.payload();

        if PAYLOAD_BITS >= i64::BITS {
            return p as i64;
        }

        let sign_bit = (1 as Word) << (PAYLOAD_BITS - 1);
        if (p & sign_bit) != 0 {
            (p | !PAYLOAD_MASK) as i64
        } else {
            p as i64
        }
    }

    #[inline]
    unsafe fn payload_to_ref<T>(&self) -> &T {
        unsafe { &*(self.payload() as usize as *const T) }
    }

    // ========================================
    // Custom Types
    // ========================================
    #[inline]
    fn from_custom(val: Box<dyn CustomDbValue>) -> Self {
        Self::from_tag_and_boxed(make_tag(CATEGORY_BOXED, TYPE_EXTENSION), val)
    }

    #[inline]
    fn as_custom<T: 'static>(&self) -> Option<&T> {
        if self.r#type() == TYPE_EXTENSION {
            let trait_obj = unsafe { self.payload_to_ref::<Box<dyn CustomDbValue>>() };
            trait_obj.as_any().downcast_ref::<T>()
        } else {
            None
        }
    }

    // ========================================
    // Nullable Types
    // ========================================

    /// Creates a `NULL` database value.
    #[inline]
    pub fn from_null() -> Self {
        Self::from_tag_and_payload(make_tag(CATEGORY_INLINE, TYPE_NULL), 0)
    }

    /// Returns `true` if this value is `NULL`.
    #[inline]
    pub fn is_null(&self) -> bool {
        self.r#type() == TYPE_NULL
    }

    // ========================================
    // Generic Conversions
    // ========================================

    /// Returns `true` if the stored type exactly matches `T`.
    #[inline]
    pub fn is<T: FromDbValue>(&self) -> bool {
        T::matches_type(self)
    }

    /// Extracts a `T` without coercion, or `None` on type mismatch.
    #[inline]
    pub fn get<T: FromDbValue>(&self) -> Option<T> {
        T::from_exact(self)
    }

    /// Extracts a `T` with best-effort coercion, or `None` if conversion is impossible.
    #[inline]
    pub fn cast<T: FromDbValue>(&self) -> Option<T> {
        T::from_cast(self)
    }
}

impl<T: ToDbValue> From<T> for DbValue {
    #[inline]
    fn from(val: T) -> Self {
        val.to_db_value()
    }
}

impl Drop for DbValue {
    fn drop(&mut self) {
        if self.category() != CATEGORY_BOXED { return; }

        let ptr = self.payload() as usize;

        unsafe  {
            match self.r#type() {
                TYPE_I64 => { let _ = Box::from_raw(ptr as *mut i64); },
                TYPE_U64 => { let _ = Box::from_raw(ptr as *mut u64); },
                TYPE_I128 => { let _ = Box::from_raw(ptr as *mut i128); },
                TYPE_U128 => { let _ = Box::from_raw(ptr as *mut u128); },
                TYPE_F64 => { let _ = Box::from_raw(ptr as *mut f64); },
                TYPE_DECIMAL => { let _ = Box::from_raw(ptr as *mut Decimal); },
                TYPE_STRING => { let _ = Box::from_raw(ptr as *mut String); },
                TYPE_DATE => { let _ = Box::from_raw(ptr as *mut NaiveDate); },
                TYPE_TIME => { let _ = Box::from_raw(ptr as *mut NaiveTime); },
                TYPE_TIMESTAMP => { let _ = Box::from_raw(ptr as *mut NaiveDateTime); },
                TYPE_TIMESTAMPZ => { let _ = Box::from_raw(ptr as *mut DateTime<Utc>); },
                TYPE_BYTES => { let _ = Box::from_raw(ptr as *mut Vec<u8>); },
                TYPE_UUID => { let _ = Box::from_raw(ptr as *mut Uuid); },
                TYPE_JSON => { let _ = Box::from_raw(ptr as *mut JsonValue); },
                TYPE_EXTENSION => {
                    let _ = Box::from_raw(ptr as *mut Box<dyn CustomDbValue>);
                },
                _ => {}
            }
        }
    }
}

impl Clone for DbValue {
    fn clone(&self) -> Self {
        if self.category() != CATEGORY_BOXED { return Self(self.0); }
        match self.r#type() {
            TYPE_I64 => DbValue::from_tag_and_boxed(make_tag(CATEGORY_BOXED, TYPE_I64), unsafe { *self.payload_to_ref::<i64>() }),
            TYPE_U64 => DbValue::from_tag_and_boxed(make_tag(CATEGORY_BOXED, TYPE_U64), unsafe { *self.payload_to_ref::<u64>() }),
            TYPE_I128 => DbValue::from_tag_and_boxed(make_tag(CATEGORY_BOXED, TYPE_I128), unsafe { *self.payload_to_ref::<i128>() }),
            TYPE_U128 => DbValue::from_tag_and_boxed(make_tag(CATEGORY_BOXED, TYPE_U128), unsafe { *self.payload_to_ref::<u128>() }),
            TYPE_F64 => DbValue::from_tag_and_boxed(make_tag(CATEGORY_BOXED, TYPE_F64), unsafe { *self.payload_to_ref::<f64>() }),
            TYPE_DATE => DbValue::from_tag_and_boxed(make_tag(CATEGORY_BOXED, TYPE_DATE), unsafe { *self.payload_to_ref::<NaiveDate>() }),
            TYPE_TIME => DbValue::from_tag_and_boxed(make_tag(CATEGORY_BOXED, TYPE_TIME), unsafe { *self.payload_to_ref::<NaiveTime>() }),
            TYPE_TIMESTAMP => DbValue::from_tag_and_boxed(make_tag(CATEGORY_BOXED, TYPE_TIMESTAMP), unsafe { *self.payload_to_ref::<NaiveDateTime>() }),
            TYPE_UUID => DbValue::from_tag_and_boxed(make_tag(CATEGORY_BOXED, TYPE_UUID), unsafe { *self.payload_to_ref::<Uuid>() }),
            TYPE_DECIMAL => DbValue::from_tag_and_boxed(make_tag(CATEGORY_BOXED, TYPE_DECIMAL), unsafe { self.payload_to_ref::<Decimal>() }.clone()),
            TYPE_STRING => DbValue::from_tag_and_boxed(make_tag(CATEGORY_BOXED, TYPE_STRING), unsafe { self.payload_to_ref::<String>() }.clone()),
            TYPE_TIMESTAMPZ => DbValue::from_tag_and_boxed(make_tag(CATEGORY_BOXED, TYPE_TIMESTAMPZ), unsafe { self.payload_to_ref::<DateTime<Utc>>() }.clone()),
            TYPE_BYTES => DbValue::from_tag_and_boxed(make_tag(CATEGORY_BOXED, TYPE_BYTES), unsafe { self.payload_to_ref::<Vec<u8>>() }.clone()),
            TYPE_JSON => DbValue::from_tag_and_boxed(make_tag(CATEGORY_BOXED, TYPE_JSON), unsafe { self.payload_to_ref::<JsonValue>() }.clone()),
            TYPE_EXTENSION => {
                let original = unsafe { self.payload_to_ref::<Box<dyn CustomDbValue>>() };
                let cloned_trait = original.clone_box();
                DbValue::from_custom(cloned_trait)
            }
            _ => Self(self.0),
        }
    }
}

impl PartialEq for DbValue {
    fn eq(&self, other: &Self) -> bool {
        if self.r#type() != other.r#type() { return false; }
        match self.r#type() {
            TYPE_NULL => true,
            TYPE_BOOL => self.get::<bool>() == other.get::<bool>(),
            TYPE_I8 => self.get::<i8>() == other.get::<i8>(),
            TYPE_I16 => self.get::<i16>() == other.get::<i16>(),
            TYPE_I32 => self.get::<i32>() == other.get::<i32>(),
            TYPE_I64 => self.get::<i64>() == other.get::<i64>(),
            TYPE_I128 => self.get::<i128>() == other.get::<i128>(),
            TYPE_U8 => self.get::<u8>() == other.get::<u8>(),
            TYPE_U16 => self.get::<u16>() == other.get::<u16>(),
            TYPE_U32 => self.get::<u32>() == other.get::<u32>(),
            TYPE_U64 => self.get::<u64>() == other.get::<u64>(),
            TYPE_U128 => self.get::<u128>() == other.get::<u128>(),
            TYPE_F32 => self.get::<f32>() == other.get::<f32>(),
            TYPE_F64 => self.get::<f64>() == other.get::<f64>(),
            TYPE_DECIMAL => self.get::<Decimal>() == other.get::<Decimal>(),
            TYPE_CHAR => self.get::<char>() == other.get::<char>(),
            TYPE_STRING => self.get::<String>() == other.get::<String>(),
            TYPE_DATE => self.get::<NaiveDate>() == other.get::<NaiveDate>(),
            TYPE_TIME => self.get::<NaiveTime>() == other.get::<NaiveTime>(),
            TYPE_TIMESTAMP => self.get::<NaiveDateTime>() == other.get::<NaiveDateTime>(),
            TYPE_TIMESTAMPZ => self.get::<DateTime<Utc>>() == other.get::<DateTime<Utc>>(),
            TYPE_BYTES => self.get::<Vec<u8>>() == other.get::<Vec<u8>>(),
            TYPE_UUID => self.get::<Uuid>() == other.get::<Uuid>(),
            TYPE_JSON => self.get::<JsonValue>() == other.get::<JsonValue>(),
            TYPE_EXTENSION => {
                let self_ext = unsafe { self.payload_to_ref::<Box<dyn CustomDbValue>>() };
                let other_ext = unsafe { other.payload_to_ref::<Box<dyn CustomDbValue>>() };
                self_ext.dyn_eq(other_ext.as_ref())
            }
            _ => false,
        }
    }
}

impl PartialOrd for DbValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.r#type() != other.r#type() { return None; }
        match self.r#type() {
            TYPE_NULL => Some(std::cmp::Ordering::Equal),
            TYPE_BOOL => self.get::<bool>().partial_cmp(&other.get::<bool>()),
            TYPE_I8 => self.get::<i8>().partial_cmp(&other.get::<i8>()),
            TYPE_I16 => self.get::<i16>().partial_cmp(&other.get::<i16>()),
            TYPE_I32 => self.get::<i32>().partial_cmp(&other.get::<i32>()),
            TYPE_I64 => self.get::<i64>().partial_cmp(&other.get::<i64>()),
            TYPE_I128 => self.get::<i128>().partial_cmp(&other.get::<i128>()),
            TYPE_U8 => self.get::<u8>().partial_cmp(&other.get::<u8>()),
            TYPE_U16 => self.get::<u16>().partial_cmp(&other.get::<u16>()),
            TYPE_U32 => self.get::<u32>().partial_cmp(&other.get::<u32>()),
            TYPE_U64 => self.get::<u64>().partial_cmp(&other.get::<u64>()),
            TYPE_U128 => self.get::<u128>().partial_cmp(&other.get::<u128>()),
            TYPE_F32 => self.get::<f32>().partial_cmp(&other.get::<f32>()),
            TYPE_F64 => self.get::<f64>().partial_cmp(&other.get::<f64>()),
            TYPE_DECIMAL => self.get::<Decimal>().partial_cmp(&other.get::<Decimal>()),
            TYPE_CHAR => self.get::<char>().partial_cmp(&other.get::<char>()),
            TYPE_STRING => self.get::<String>().partial_cmp(&other.get::<String>()),
            TYPE_DATE => self.get::<NaiveDate>().partial_cmp(&other.get::<NaiveDate>()),
            TYPE_TIME => self.get::<NaiveTime>().partial_cmp(&other.get::<NaiveTime>()),
            TYPE_TIMESTAMP => self.get::<NaiveDateTime>().partial_cmp(&other.get::<NaiveDateTime>()),
            TYPE_TIMESTAMPZ => self.get::<DateTime<Utc>>().partial_cmp(&other.get::<DateTime<Utc>>()),
            TYPE_BYTES => self.get::<Vec<u8>>().partial_cmp(&other.get::<Vec<u8>>()),
            TYPE_UUID => self.get::<Uuid>().partial_cmp(&other.get::<Uuid>()),
            TYPE_JSON => None,
            TYPE_EXTENSION => {
                let self_ext = unsafe { self.payload_to_ref::<Box<dyn CustomDbValue>>() };
                let other_ext = unsafe { other.payload_to_ref::<Box<dyn CustomDbValue>>() };
                self_ext.dyn_partial_cmp(other_ext.as_ref())
            }
            _ => None,
        }
    }
}

// ========================================
// Implementations of ToDbValue
// ========================================
impl ToDbValue for bool {
    fn to_db_value(&self) -> DbValue {
        DbValue::from_tag_and_payload(make_tag(CATEGORY_INLINE, TYPE_BOOL), *self as Word)
    }
}

impl ToDbValue for i8 {
    fn to_db_value(&self) -> DbValue {
        DbValue::from_tag_and_payload(make_tag(CATEGORY_INLINE, TYPE_I8), (*self as u8) as Word)
    }
}

impl ToDbValue for i16 {
    fn to_db_value(&self) -> DbValue {
        DbValue::from_tag_and_payload(make_tag(CATEGORY_INLINE, TYPE_I16), (*self as u16) as Word)
    }
}

impl ToDbValue for i32 {
    fn to_db_value(&self) -> DbValue {
        DbValue::from_tag_and_payload(make_tag(CATEGORY_INLINE, TYPE_I32), (*self as u32) as Word)
    }
}

impl ToDbValue for i64 {
    fn to_db_value(&self) -> DbValue {
        if PAYLOAD_BITS >= i64::BITS {
            return DbValue::from_tag_and_payload(make_tag(CATEGORY_INLINE, TYPE_I64), *self as Word);
        }

        let min_inline = -(1i64 << (PAYLOAD_BITS - 1));
        let max_inline = (1i64 << (PAYLOAD_BITS - 1)) - 1;

        if (min_inline..=max_inline).contains(self) {
            // Mask to payload width so the tag bits are clean for negative values
            DbValue::from_tag_and_payload(make_tag(CATEGORY_INLINE, TYPE_I64), (*self as Word) & PAYLOAD_MASK)
        } else {
            DbValue::from_tag_and_boxed(make_tag(CATEGORY_BOXED, TYPE_I64), *self)
        }
    }
}

impl ToDbValue for i128 {
    fn to_db_value(&self) -> DbValue {
        let min_inline = -(1i128 << (PAYLOAD_BITS - 1));
        let max_inline = (1i128 << (PAYLOAD_BITS - 1)) - 1;

        if (min_inline..=max_inline).contains(self) {
            // Mask to payload width so the tag bits are clean for negative values
            DbValue::from_tag_and_payload(make_tag(CATEGORY_INLINE, TYPE_I128), (*self as Word) & PAYLOAD_MASK)
        } else {
            DbValue::from_tag_and_boxed(make_tag(CATEGORY_BOXED, TYPE_I128), *self)
        }
    }
}

impl ToDbValue for u8 {
    fn to_db_value(&self) -> DbValue {
        DbValue::from_tag_and_payload(make_tag(CATEGORY_INLINE, TYPE_U8), *self as Word)
    }
}

impl ToDbValue for u16 {
    fn to_db_value(&self) -> DbValue {
        DbValue::from_tag_and_payload(make_tag(CATEGORY_INLINE, TYPE_U16), *self as Word)
    }
}

impl ToDbValue for u32 {
    fn to_db_value(&self) -> DbValue {
        DbValue::from_tag_and_payload(make_tag(CATEGORY_INLINE, TYPE_U32), *self as Word)
    }
}

impl ToDbValue for u64 {
    fn to_db_value(&self) -> DbValue {
        if PAYLOAD_BITS >= u64::BITS {
            return DbValue::from_tag_and_payload(make_tag(CATEGORY_INLINE, TYPE_U64), *self as Word);
        }

        let max_inline = (1u64 << PAYLOAD_BITS) - 1;

        if *self <= max_inline {
            DbValue::from_tag_and_payload(make_tag(CATEGORY_INLINE, TYPE_U64), *self as Word)
        } else {
            DbValue::from_tag_and_boxed(make_tag(CATEGORY_BOXED, TYPE_U64), *self)
        }
    }
}

impl ToDbValue for u128 {
    fn to_db_value(&self) -> DbValue {
        let max_inline = (1u128 << PAYLOAD_BITS) - 1;

        if *self <= max_inline {
            DbValue::from_tag_and_payload(make_tag(CATEGORY_INLINE, TYPE_U128), *self as Word)
        } else {
            DbValue::from_tag_and_boxed(make_tag(CATEGORY_BOXED, TYPE_U128), *self)
        }
    }
}

impl ToDbValue for f32 {
    fn to_db_value(&self) -> DbValue {
        DbValue::from_tag_and_payload(make_tag(CATEGORY_INLINE, TYPE_F32), (*self).to_bits() as Word)
    }
}

impl ToDbValue for f64 {
    fn to_db_value(&self) -> DbValue {
        if PAYLOAD_BITS >= u64::BITS {
            DbValue::from_tag_and_payload(make_tag(CATEGORY_INLINE, TYPE_F64), self.to_bits() as Word)
        } else {
            DbValue::from_tag_and_boxed(make_tag(CATEGORY_BOXED, TYPE_F64), *self)
        }
    }
}

impl ToDbValue for Decimal {
    fn to_db_value(&self) -> DbValue {
        DbValue::from_tag_and_boxed(make_tag(CATEGORY_BOXED, TYPE_DECIMAL), *self)
    }
}

impl ToDbValue for char {
    fn to_db_value(&self) -> DbValue {
        DbValue::from_tag_and_payload(make_tag(CATEGORY_INLINE, TYPE_CHAR), *self as Word)
    }
}

impl ToDbValue for String {
    fn to_db_value(&self) -> DbValue {
        DbValue::from_tag_and_boxed(make_tag(CATEGORY_BOXED, TYPE_STRING), self.clone())
    }
}

impl ToDbValue for &str {
    fn to_db_value(&self) -> DbValue {
        DbValue::from_tag_and_boxed(make_tag(CATEGORY_BOXED, TYPE_STRING), (*self).to_string())
    }
}

impl ToDbValue for SmolStr {
    fn to_db_value(&self) -> DbValue {
        DbValue::from_tag_and_boxed(make_tag(CATEGORY_BOXED, TYPE_STRING), self.to_string())
    }
}

impl ToDbValue for NaiveDate {
    fn to_db_value(&self) -> DbValue {
        DbValue::from_tag_and_boxed(make_tag(CATEGORY_BOXED, TYPE_DATE), *self)
    }
}

impl ToDbValue for NaiveTime {
    fn to_db_value(&self) -> DbValue {
        DbValue::from_tag_and_boxed(make_tag(CATEGORY_BOXED, TYPE_TIME), *self)
    }
}

impl ToDbValue for NaiveDateTime {
    fn to_db_value(&self) -> DbValue {
        DbValue::from_tag_and_boxed(make_tag(CATEGORY_BOXED, TYPE_TIMESTAMP), *self)
    }
}

impl ToDbValue for DateTime<Utc> {
    fn to_db_value(&self) -> DbValue {
        DbValue::from_tag_and_boxed(make_tag(CATEGORY_BOXED, TYPE_TIMESTAMPZ), *self)
    }
}

impl ToDbValue for Vec<u8> {
    fn to_db_value(&self) -> DbValue {
        DbValue::from_tag_and_boxed(make_tag(CATEGORY_BOXED, TYPE_BYTES), self.clone())
    }
}

impl ToDbValue for Uuid {
    fn to_db_value(&self) -> DbValue {
        DbValue::from_tag_and_boxed(make_tag(CATEGORY_BOXED, TYPE_UUID), *self)
    }
}

impl ToDbValue for JsonValue {
    fn to_db_value(&self) -> DbValue {
        DbValue::from_tag_and_boxed(make_tag(CATEGORY_BOXED, TYPE_JSON), self.clone())
    }
}

impl<T: CustomDbValue + Clone> ToDbValue for T {
    #[inline]
    fn to_db_value(&self) -> DbValue {
        let boxed_trait: Box<dyn CustomDbValue> = Box::new(self.clone());
        DbValue::from_custom(boxed_trait)
    }
}

// ========================================
// Implementations of FromDbValue
// ========================================
impl FromDbValue for bool {
    fn matches_type(value: &DbValue) -> bool {
        value.r#type() == TYPE_BOOL
    }

    fn from_exact(value: &DbValue) -> Option<Self> {
        Self::matches_type(value).then(|| value.payload() != 0)
    }

    fn from_cast(value: &DbValue) -> Option<Self> {
        Self::from_exact(value)
    }
}

impl FromDbValue for i8 {
    fn matches_type(value: &DbValue) -> bool {
        value.r#type() == TYPE_I8
    }

    fn from_exact(value: &DbValue) -> Option<Self> {
        Self::matches_type(value).then(|| value.payload() as i8)
    }

    fn from_cast(value: &DbValue) -> Option<Self> {
        Self::from_exact(value)
    }
}

impl FromDbValue for i16 {
    fn matches_type(value: &DbValue) -> bool {
        value.r#type() == TYPE_I16
    }

    fn from_exact(value: &DbValue) -> Option<Self> {
        Self::matches_type(value).then(|| value.payload() as i16)
    }

    fn from_cast(value: &DbValue) -> Option<Self> {
        Self::from_exact(value)
    }
}

impl FromDbValue for i32 {
    fn matches_type(value: &DbValue) -> bool {
        value.r#type() == TYPE_I32
    }

    fn from_exact(value: &DbValue) -> Option<Self> {
        Self::matches_type(value).then(|| value.payload() as i32)
    }

    fn from_cast(value: &DbValue) -> Option<Self> {
        Self::from_exact(value)
    }
}

impl FromDbValue for i64 {
    fn matches_type(value: &DbValue) -> bool {
        value.r#type() == TYPE_I64
    }

    fn from_exact(value: &DbValue) -> Option<Self> {
        if !Self::matches_type(value) {
            None
        } else if value.category() == CATEGORY_INLINE {
            Some(value.payload_to_i64_i48())
        } else {
            unsafe { Some(*value.payload_to_ref::<i64>()) }
        }
    }

    fn from_cast(value: &DbValue) -> Option<Self> {
        Self::from_exact(value)
    }
}

impl FromDbValue for i128 {
    fn matches_type(value: &DbValue) -> bool {
        value.r#type() == TYPE_I128
    }

    fn from_exact(value: &DbValue) -> Option<Self> {
        if !Self::matches_type(value) {
            None
        } else if value.category() == CATEGORY_INLINE {
            // Inline i128 values fit in the same signed 47-bit range as inline i64;
            // reuse the sign-extension helper and widen to i128.
            Some(value.payload_to_i64_i48() as i128)
        } else {
            unsafe { Some(*value.payload_to_ref::<i128>()) }
        }
    }

    fn from_cast(value: &DbValue) -> Option<Self> {
        Self::from_exact(value)
    }
}

impl FromDbValue for u8 {
    fn matches_type(value: &DbValue) -> bool {
        value.r#type() == TYPE_U8
    }

    fn from_exact(value: &DbValue) -> Option<Self> {
        Self::matches_type(value).then(|| value.payload() as u8)
    }

    fn from_cast(value: &DbValue) -> Option<Self> {
        Self::from_exact(value)
    }
}

impl FromDbValue for u16 {
    fn matches_type(value: &DbValue) -> bool {
        value.r#type() == TYPE_U16
    }

    fn from_exact(value: &DbValue) -> Option<Self> {
        Self::matches_type(value).then(|| value.payload() as u16)
    }

    fn from_cast(value: &DbValue) -> Option<Self> {
        Self::from_exact(value)
    }
}

impl FromDbValue for u32 {
    fn matches_type(value: &DbValue) -> bool {
        value.r#type() == TYPE_U32
    }

    fn from_exact(value: &DbValue) -> Option<Self> {
        Self::matches_type(value).then(|| value.payload() as u32)
    }

    fn from_cast(value: &DbValue) -> Option<Self> {
        Self::from_exact(value)
    }
}

impl FromDbValue for u64 {
    fn matches_type(value: &DbValue) -> bool {
        value.r#type() == TYPE_U64
    }

    fn from_exact(value: &DbValue) -> Option<Self> {
        if !Self::matches_type(value) {
            None
        } else if value.category() == CATEGORY_INLINE {
            Some(value.payload() as u64)
        } else {
            unsafe { Some(*value.payload_to_ref::<u64>()) }
        }
    }

    fn from_cast(value: &DbValue) -> Option<Self> {
        Self::from_exact(value)
    }
}

impl FromDbValue for u128 {
    fn matches_type(value: &DbValue) -> bool {
        value.r#type() == TYPE_U128
    }

    fn from_exact(value: &DbValue) -> Option<Self> {
        if !Self::matches_type(value) {
            None
        } else if value.category() == CATEGORY_INLINE {
            Some(value.payload() as u128)
        } else {
            unsafe { Some(*value.payload_to_ref::<u128>()) }
        }
    }

    fn from_cast(value: &DbValue) -> Option<Self> {
        Self::from_exact(value)
    }
}

impl FromDbValue for f32 {
    fn matches_type(value: &DbValue) -> bool {
        value.r#type() == TYPE_F32
    }

    fn from_exact(value: &DbValue) -> Option<Self> {
        Self::matches_type(value).then(|| f32::from_bits(value.payload() as u32))
    }

    fn from_cast(value: &DbValue) -> Option<Self> {
        Self::from_exact(value)
    }
}

impl FromDbValue for f64 {
    fn matches_type(value: &DbValue) -> bool {
        value.r#type() == TYPE_F64
    }

    fn from_exact(value: &DbValue) -> Option<Self> {
        if !Self::matches_type(value) {
            None
        } else if value.category() == CATEGORY_INLINE {
            Some(f64::from_bits(value.payload() as u64))
        } else {
            unsafe { Some(*value.payload_to_ref::<f64>()) }
        }
    }

    fn from_cast(value: &DbValue) -> Option<Self> {
        Self::from_exact(value)
    }
}

impl FromDbValue for Decimal {
    fn matches_type(value: &DbValue) -> bool {
        value.r#type() == TYPE_DECIMAL
    }

    fn from_exact(value: &DbValue) -> Option<Self> {
        Self::matches_type(value).then(|| unsafe { *value.payload_to_ref::<Decimal>() })
    }

    fn from_cast(value: &DbValue) -> Option<Self> {
        Self::from_exact(value)
    }
}

impl FromDbValue for char {
    fn matches_type(value: &DbValue) -> bool {
        value.r#type() == TYPE_CHAR
    }

    fn from_exact(value: &DbValue) -> Option<Self> {
        Self::matches_type(value).then(|| std::char::from_u32(value.payload() as u32)).flatten()
    }

    fn from_cast(value: &DbValue) -> Option<Self> {
        Self::from_exact(value)
    }
}

impl FromDbValue for String {
    fn matches_type(value: &DbValue) -> bool {
        value.r#type() == TYPE_STRING
    }

    fn from_exact(value: &DbValue) -> Option<Self> {
        Self::matches_type(value).then(|| unsafe { (*value.payload_to_ref::<String>()).clone() })
    }

    fn from_cast(value: &DbValue) -> Option<Self> {
        Self::from_exact(value)
    }
}

impl FromDbValue for NaiveDate {
    fn matches_type(value: &DbValue) -> bool {
        value.r#type() == TYPE_DATE
    }

    fn from_exact(value: &DbValue) -> Option<Self> {
        Self::matches_type(value).then(|| unsafe { *value.payload_to_ref::<NaiveDate>() })
    }

    fn from_cast(value: &DbValue) -> Option<Self> {
        Self::from_exact(value)
    }
}

impl FromDbValue for NaiveTime {
    fn matches_type(value: &DbValue) -> bool {
        value.r#type() == TYPE_TIME
    }

    fn from_exact(value: &DbValue) -> Option<Self> {
        Self::matches_type(value).then(|| unsafe { *value.payload_to_ref::<NaiveTime>() })
    }

    fn from_cast(value: &DbValue) -> Option<Self> {
        Self::from_exact(value)
    }
}

impl FromDbValue for NaiveDateTime {
    fn matches_type(value: &DbValue) -> bool {
        value.r#type() == TYPE_TIMESTAMP
    }

    fn from_exact(value: &DbValue) -> Option<Self> {
        Self::matches_type(value).then(|| unsafe { *value.payload_to_ref::<NaiveDateTime>() })
    }

    fn from_cast(value: &DbValue) -> Option<Self> {
        Self::from_exact(value)
    }
}

impl FromDbValue for DateTime<Utc> {
    fn matches_type(value: &DbValue) -> bool {
        value.r#type() == TYPE_TIMESTAMPZ
    }

    fn from_exact(value: &DbValue) -> Option<Self> {
        Self::matches_type(value).then(|| unsafe { *value.payload_to_ref::<DateTime<Utc>>() })
    }

    fn from_cast(value: &DbValue) -> Option<Self> {
        Self::from_exact(value)
    }
}

impl FromDbValue for Vec<u8> {
    fn matches_type(value: &DbValue) -> bool {
        value.r#type() == TYPE_BYTES
    }

    fn from_exact(value: &DbValue) -> Option<Self> {
        Self::matches_type(value).then(|| unsafe { (*value.payload_to_ref::<Vec<u8>>()).clone() })
    }

    fn from_cast(value: &DbValue) -> Option<Self> {
        Self::from_exact(value)
    }
}

impl FromDbValue for Uuid {
    fn matches_type(value: &DbValue) -> bool {
        value.r#type() == TYPE_UUID
    }

    fn from_exact(value: &DbValue) -> Option<Self> {
        Self::matches_type(value).then(|| unsafe { *value.payload_to_ref::<Uuid>() })
    }

    fn from_cast(value: &DbValue) -> Option<Self> {
        Self::from_exact(value)
    }
}

impl FromDbValue for JsonValue {
    fn matches_type(value: &DbValue) -> bool {
        value.r#type() == TYPE_JSON
    }

    fn from_exact(value: &DbValue) -> Option<Self> {
        Self::matches_type(value).then(|| unsafe { (*value.payload_to_ref::<JsonValue>()).clone() })
    }

    fn from_cast(value: &DbValue) -> Option<Self> {
        Self::from_exact(value)
    }
}

impl<T: CustomDbValue + Clone> FromDbValue for T {
    #[inline]
    fn matches_type(value: &DbValue) -> bool {
        value.r#type() == TYPE_EXTENSION && value.as_custom::<Self>().is_some()
    }

    #[inline]
    fn from_exact(value: &DbValue) -> Option<Self> {
        value.as_custom::<Self>().cloned()
    }

    #[inline]
    fn from_cast(value: &DbValue) -> Option<Self> {
        Self::from_exact(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_roundtrip() {
        let v = DbValue::from_null();
        assert!(v.is_null());
        assert!(!v.is::<bool>());
        assert_eq!(v.get::<bool>(), None);
        assert_eq!(DbValue::from_null(), DbValue::from_null());
    }

    #[test]
    fn bool_roundtrip() {
        let t = true.to_db_value();
        let f = false.to_db_value();
        assert_eq!(t.get::<bool>(), Some(true));
        assert_eq!(f.get::<bool>(), Some(false));
        assert!(t.is::<bool>());
        assert!(!t.is_null());
    }

    #[test]
    fn i8_roundtrip() {
        assert_eq!(0i8.to_db_value().get::<i8>(), Some(0i8));
        assert_eq!(127i8.to_db_value().get::<i8>(), Some(127i8));
        assert_eq!((-128i8).to_db_value().get::<i8>(), Some(-128i8));
    }

    #[test]
    fn i16_roundtrip() {
        assert_eq!(1000i16.to_db_value().get::<i16>(), Some(1000i16));
        assert_eq!((-30000i16).to_db_value().get::<i16>(), Some(-30000i16));
        assert_eq!(i16::MAX.to_db_value().get::<i16>(), Some(i16::MAX));
        assert_eq!(i16::MIN.to_db_value().get::<i16>(), Some(i16::MIN));
    }

    #[test]
    fn i32_roundtrip() {
        assert_eq!(1_000_000i32.to_db_value().get::<i32>(), Some(1_000_000i32));
        assert_eq!((-1_000_000i32).to_db_value().get::<i32>(), Some(-1_000_000i32));
        assert_eq!(i32::MAX.to_db_value().get::<i32>(), Some(i32::MAX));
        assert_eq!(i32::MIN.to_db_value().get::<i32>(), Some(i32::MIN));
    }

    #[test]
    fn i64_roundtrip_inline_values() {
        // All values within the 47-bit signed inline range (no heap allocation needed)
        assert_eq!(0i64.to_db_value().get::<i64>(), Some(0i64));
        assert_eq!(42i64.to_db_value().get::<i64>(), Some(42i64));
        assert_eq!((-42i64).to_db_value().get::<i64>(), Some(-42i64));
        assert_eq!(1_000_000_000i64.to_db_value().get::<i64>(), Some(1_000_000_000i64));
        assert_eq!((-1_000_000_000i64).to_db_value().get::<i64>(), Some(-1_000_000_000i64));
        // Boundary of inline range (2^47 - 1 and -(2^47))
        let max_inline: i64 = (1i64 << 47) - 1;
        let min_inline: i64 = -(1i64 << 47);
        assert_eq!(max_inline.to_db_value().get::<i64>(), Some(max_inline));
        assert_eq!(min_inline.to_db_value().get::<i64>(), Some(min_inline));
    }

    #[test]
    fn i128_roundtrip() {
        // Inline path (small values within the 47-bit signed range)
        assert_eq!(42i128.to_db_value().get::<i128>(), Some(42i128));
        assert_eq!((-42i128).to_db_value().get::<i128>(), Some(-42i128));
        assert_eq!(0i128.to_db_value().get::<i128>(), Some(0i128));
    }

    #[test]
    fn u8_roundtrip() {
        assert_eq!(0u8.to_db_value().get::<u8>(), Some(0u8));
        assert_eq!(u8::MAX.to_db_value().get::<u8>(), Some(u8::MAX));
    }

    #[test]
    fn u16_roundtrip() {
        assert_eq!(u16::MAX.to_db_value().get::<u16>(), Some(u16::MAX));
        assert_eq!(0u16.to_db_value().get::<u16>(), Some(0u16));
    }

    #[test]
    fn u32_roundtrip() {
        assert_eq!(u32::MAX.to_db_value().get::<u32>(), Some(u32::MAX));
        assert_eq!(0u32.to_db_value().get::<u32>(), Some(0u32));
    }

    #[test]
    fn u64_roundtrip_inline_and_boxed() {
        assert_eq!(0u64.to_db_value().get::<u64>(), Some(0u64));
        assert_eq!(u64::MAX.to_db_value().get::<u64>(), Some(u64::MAX));
        assert_eq!(42u64.to_db_value().get::<u64>(), Some(42u64));
    }

    #[test]
    fn u128_roundtrip() {
        assert_eq!(0u128.to_db_value().get::<u128>(), Some(0u128));
        assert_eq!(u128::MAX.to_db_value().get::<u128>(), Some(u128::MAX));
    }

    #[test]
    fn f32_roundtrip() {
        let v = 3.14f32;
        let got = v.to_db_value().get::<f32>().unwrap();
        assert!((got - v).abs() < f32::EPSILON);
        assert_eq!(0f32.to_db_value().get::<f32>(), Some(0f32));
    }

    #[test]
    fn f64_roundtrip() {
        let v = std::f64::consts::PI;
        let got = v.to_db_value().get::<f64>().unwrap();
        assert!((got - v).abs() < f64::EPSILON);
        assert_eq!(0f64.to_db_value().get::<f64>(), Some(0f64));
    }

    #[test]
    fn char_roundtrip() {
        assert_eq!('A'.to_db_value().get::<char>(), Some('A'));
        assert_eq!('€'.to_db_value().get::<char>(), Some('€'));
        assert_eq!('\0'.to_db_value().get::<char>(), Some('\0'));
    }

    #[test]
    fn string_roundtrip() {
        let s = "hello world".to_string();
        assert_eq!(s.to_db_value().get::<String>(), Some(s));
        assert_eq!("".to_string().to_db_value().get::<String>(), Some(String::new()));
    }

    #[test]
    fn str_roundtrip() {
        let v = "hello".to_db_value();
        assert_eq!(v.get::<String>(), Some("hello".to_string()));
    }

    #[test]
    fn bytes_roundtrip() {
        let bytes = vec![0u8, 1, 127, 255];
        assert_eq!(bytes.clone().to_db_value().get::<Vec<u8>>(), Some(bytes));
        assert_eq!(vec![].to_db_value().get::<Vec<u8>>(), Some(vec![]));
    }

    #[test]
    fn uuid_roundtrip() {
        let id = Uuid::nil();
        assert_eq!(id.to_db_value().get::<Uuid>(), Some(id));
    }

    #[test]
    fn clone_is_independent() {
        let original = "hello".to_db_value();
        let cloned = original.clone();
        assert_eq!(original, cloned);
        assert_eq!(original.get::<String>(), Some("hello".to_string()));
        assert_eq!(cloned.get::<String>(), Some("hello".to_string()));

        let n = 42i64.to_db_value();
        let n2 = n.clone();
        assert_eq!(n, n2);
        assert_eq!(n2.get::<i64>(), Some(42i64));
    }

    #[test]
    fn type_mismatch_returns_none() {
        let v = 42i32.to_db_value();
        assert_eq!(v.get::<i64>(), None);
        assert_eq!(v.get::<String>(), None);
        assert_eq!(v.get::<bool>(), None);
        assert!(!v.is::<i64>());
    }

    #[test]
    fn partial_eq_same_value_same_type() {
        assert_eq!(42i32.to_db_value(), 42i32.to_db_value());
        assert_ne!(42i32.to_db_value(), 43i32.to_db_value());
        assert_eq!("hello".to_db_value(), "hello".to_db_value());
        assert_ne!("hello".to_db_value(), "world".to_db_value());
        assert_eq!(DbValue::from_null(), DbValue::from_null());
    }

    #[test]
    fn partial_eq_different_types_always_false() {
        // Same numeric value but different integer types must NOT be equal
        assert_ne!(42i32.to_db_value(), 42i64.to_db_value());
        assert_ne!(42u32.to_db_value(), 42i32.to_db_value());
    }

    #[test]
    fn partial_ord_integers() {
        let a = 10i32.to_db_value();
        let b = 20i32.to_db_value();
        assert!(a < b);
        assert!(b > a);
        assert_eq!(a.partial_cmp(&a), Some(std::cmp::Ordering::Equal));
    }

    #[test]
    fn partial_ord_different_types_is_none() {
        assert_eq!(10i32.to_db_value().partial_cmp(&10i64.to_db_value()), None);
    }

    #[test]
    fn from_into_conversion() {
        let v: DbValue = 99i32.into();
        assert_eq!(v.get::<i32>(), Some(99i32));
    }

    #[test]
    fn is_type_checks() {
        assert!(42i32.to_db_value().is::<i32>());
        assert!(!42i32.to_db_value().is::<i64>());
        assert!(DbValue::from_null().is_null());
        assert!(!DbValue::from_null().is::<i32>());
    }
}

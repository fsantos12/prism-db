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
pub trait ToDbValue {
    fn to_db_value(&self) -> DbValue;
}

// ========================================
// 2. Deserialization (DB -> Rust)
// ========================================
pub trait FromDbValue: Sized {
    /// Extremely fast, zero-overhead exact type check
    fn matches_type(value: &DbValue) -> bool;

    /// Strict extraction. Returns None if the tag doesn't match perfectly.
    fn from_exact(value: &DbValue) -> Option<Self>;

    /// Loose extraction. Parses strings, checks integer bounds, etc.
    fn from_cast(value: &DbValue) -> Option<Self>;
}

// ========================================
// 3. The Custom Extension Trait
// ========================================
pub trait CustomDbValue: Send + Sync + 'static {
    // Required so the database can clone rows in memory
    fn clone_box(&self) -> Box<dyn CustomDbValue>;

    // Required for runtime downcasting back to the user's struct
    fn as_any(&self) -> &dyn Any;

    fn dyn_eq(&self, other: &dyn CustomDbValue) -> bool;

    fn dyn_partial_cmp(&self, other: &dyn CustomDbValue) -> Option<std::cmp::Ordering>;
}

#[inline]
fn make_tag(category: Word, r#type: Word) -> Word {
    debug_assert_eq!(category & !CATEGORY_MASK, 0);
    debug_assert_eq!(r#type & !TYPE_MASK, 0);
    category | r#type
}

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
    #[inline]
    pub fn from_null() -> Self {
        Self::from_tag_and_payload(make_tag(CATEGORY_INLINE, TYPE_NULL), 0)
    }

    #[inline]
    pub fn is_null(&self) -> bool {
        self.r#type() == TYPE_NULL
    }

    // ========================================
    // Generic Conversions
    // ========================================
    #[inline]
    pub fn is<T: FromDbValue>(&self) -> bool {
        T::matches_type(self)
    }

    #[inline]
    pub fn get<T: FromDbValue>(&self) -> Option<T> {
        T::from_exact(self)
    }

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
            DbValue::from_tag_and_payload(make_tag(CATEGORY_INLINE, TYPE_I64), *self as Word)
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
            DbValue::from_tag_and_payload(make_tag(CATEGORY_INLINE, TYPE_I128), *self as Word)
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
            Some(value.payload() as i128)
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
use std::time::{Instant, Duration};
use bnum::cast::{As, CastFrom};
use bnum::BUintD8;
use rand::prelude::*;
use rand_distr::StandardNormal;

const HALF_LIFE: f64 = 703_800_000.0;
const RADIATION_REACH: usize = 2 * 8;


#[allow(non_camel_case_types)]
#[repr(C)]
pub struct u235 {
	value: BUintD8<30>,
	updated: Instant,
}

struct Hazmat<const P: usize, T: std::default::Default> {
	pad1: [u8; P],
	item: T,
	pad2: [u8; P],
}

impl<const P: usize, T: std::default::Default> Hazmat<P, T> {
	fn contain(&mut self, thing: T) {
		self.item = thing;
	}

	fn item(&mut self) -> &mut T {
		&mut self.item
	}
}

type OkHazmat<T> = Hazmat<RADIATION_REACH, T>;

struct HazmatManufacturer;
impl HazmatManufacturer {
	fn ok_hazmat<T: std::default::Default>() -> OkHazmat<T> {
		Hazmat {
			pad1: [0; RADIATION_REACH],
			item: T::default(),
			pad2: [0; RADIATION_REACH],
		}
	}

	fn good_hazmat<T: std::default::Default>() -> Hazmat<{2*RADIATION_REACH}, T> {
		Hazmat {
			pad1: [0; 2*RADIATION_REACH],
			item: T::default(),
			pad2: [0; 2*RADIATION_REACH],
		}
	}

	fn great_hazmat<T: std::default::Default>() -> Hazmat<{3*RADIATION_REACH}, T> {
		Hazmat {
			pad1: [0; 3*RADIATION_REACH],
			item: T::default(),
			pad2: [0; 3*RADIATION_REACH],
		}
	}
}
	
impl u235 {	
	fn max() -> u235 {
		u235 {
			value: BUintD8::from(2u64).pow(235) - BUintD8::from(1u64),
			updated: Instant::now(),
		}
	}

	fn new(val: u64) -> u235 {
		let ret = u235 {
			value: BUintD8::from(val),
			updated: Instant::now(),
		};
		if cfg!(feature = "ambient-radiation") {
			let this = &ret as *const u235 as *mut u235 as u64;
			std::thread::spawn(move || unsafe {
				loop {
					(*(this as *mut u235)).radiate();
				}
			});
		}
		std::thread::sleep(Duration::from_secs(2));
		ret
	}

	unsafe fn this(&self) -> &mut Self {
		&mut *(self as *const Self as *mut Self)
	}

	unsafe fn dump_nearby_stack(&self, range: usize, show_self: bool) {
		// let now = Instant::now();
		// let nowptr = &now as *const Instant as *mut Instant as *mut u8;
		let ptr = self as *const Self as *mut Self;
		let left = ptr.add(1) as *mut u8;
		let right = ptr as *mut u8;
		println!("{}", std::mem::size_of::<u235>());
		for i in 0..range {
			println!("{:08b}", *left.add(range-i));
		}
		println!(" (U235) ");
		if show_self {
			for i in 0..std::mem::size_of::<u235>() {
				println!("{:08b}", *right.sub(1).add(i));
			}
			println!(" (U235) ");
		}
		for i in 0..range {
			println!("{:08b}", *right.sub(i));
		}
		println!()		
	}

	unsafe fn decay(&self) {
		let now = Instant::now();
		let t = (now - self.updated).as_nanos();
		let this = self.this();
		this.value = BUintD8::cast_from(
			self.value.as_::<f64>() * (0.5f64.powf(t as f64/HALF_LIFE))
		);
		this.updated = now;
	}
	
	unsafe fn radiate(&self) {
		let ptr = self as *const Self as *mut u8;
		let val: f64 = thread_rng().sample(StandardNormal);
		let off: isize = (val * RADIATION_REACH as f64).round() as isize;
		println!("Flipping bit {off}");
		if off < 0 {			
			let shift = if (off%8).abs() == 0 { 0 } else { 8 - (off%8).abs() };
			*ptr.add(((off+1)/8) as usize) ^= 1 << shift;
		} else if off > 0 {
			let (byte, shift) = if (off%8).abs() == 0 {
				((off/8) as usize, 7)
			} else { ((off/8 + 1) as usize, (off%8).abs()-1) };
			*ptr.add(std::mem::size_of::<u235>()).add(byte) ^= 1 << shift;
		};
	}

	unsafe fn assert_value(&self, val: u64) {
		self.decay();
		assert_eq!(self.value, BUintD8::from(val))
	}

	unsafe fn to_u64(&self) -> u64 {
		self.decay();
		self.value.as_::<u64>()
	}
}

impl core::ops::Rem for u235 {
	type Output = Self;
	fn rem(self, other: Self) -> Self {
		unsafe {
			self.decay();
			Self {value: self.value % other.value, updated: self.updated}
		}
	}
}

impl core::ops::Add for u235 {
	type Output = Self;
	fn add(self, other: Self) -> Self {
		unsafe {
			self.decay();

			Self {value: (self.value + other.value) % Self::max().value, updated: self.updated}
		}
	}
}

impl core::ops::Sub for u235 {
	type Output = Self;
	fn sub(self, other: Self) -> Self {
		unsafe {
			self.decay();
			Self {value: self.value - other.value, updated: self.updated}
		}
	}
}

impl core::cmp::PartialEq for u235 {
	fn eq(&self, other: &Self) -> bool {
		unsafe {
			self.decay();
			other.decay();
			self.value == other.value
		}
	}
}

impl std::fmt::Debug for u235 {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(&format!("{:?}", self.value))
	}
}

impl std::default::Default for u235 {
	fn default() -> Self {
		u235::new(0)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::thread;
	use std::time::Duration;

	#[test]
	fn it_decays() {
		unsafe {
			let x = u235::new(16);
			assert!(x.to_u64() > 0);
			thread::sleep(Duration::from_secs(2));
			assert_eq!(x.to_u64(), 0);						
		}
	}
	
	#[test]
	fn test_hazmats() {
		let mut haz: OkHazmat<u235> = HazmatManufacturer::ok_hazmat();
		haz.contain(u235::new(10));		
	}
}

#![cfg_attr(not(feature = "std"), no_std)]

sp_api::decl_runtime_apis! {
	pub trait KittyApi
	{
		fn get_launchpad_collections() -> u32;
	}
}

#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {

	use frame_support::inherent::Vec;
	use frame_support::pallet_prelude::*;
	use frame_support::traits::Currency;
	use frame_support::{
		sp_runtime::traits::{AccountIdConversion, Zero},
		traits::{ExistenceRequirement, Randomness, ReservableCurrency, WithdrawReasons},
		PalletId,
	};
	use frame_system::pallet_prelude::*;
	use scale_info::{prelude::format, TypeInfo};

	pub type CollectionId = u32;
	pub type NFTId = u16;

	type AccountOf<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	#[cfg(feature = "std")]
	use frame_support::serde::{Deserialize, Serialize};

	// Funding
	pub type FundIndex = u32;
	const PALLET_ID: PalletId = PalletId(*b"ex/cfund");
	type FundInfoOf<T> =
		FundInfo<AccountOf<T>, BalanceOf<T>, <T as frame_system::Config>::BlockNumber>;

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	#[codec(mel_bound())]
	pub struct CollectionInfo<T: Config> {
		pub id: CollectionId,
		pub owner: AccountOf<T>,
		pub name: Vec<u8>,
		pub description: Vec<u8>,
		pub number_of_items: u16,
		pub number_of_minted: u16,
		/// The metadata of this metaverse
		// pub metadata: MetaverseMetadata,
		/// The currency use in this metaverse
		// pub currency_id: FungibleTokenId,
		/// Whether the metaverse can be transferred or not.
		pub is_frozen: bool,
		pub project_status: ProjectStatus,
		pub mint_fee: BalanceOf<T>,
		pub start_date: Option<u32>,
		pub end_date: Option<u32>,
	}

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[codec(mel_bound())]
	pub struct NFT {
		pub id: u16,
		pub name: Vec<u8>,
		pub image_url: Vec<u8>,
	}

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub enum ProjectStatus {
		Draft,
		Approved,
	}

	#[pallet::storage]
	#[pallet::getter(fn get_collections)]
	pub type Collections<T: Config> = StorageMap<_, Twox64Concat, CollectionId, CollectionInfo<T>>;

	#[pallet::storage]
	#[pallet::getter(fn get_nfts)]
	pub type NFTs<T: Config> =
		StorageDoubleMap<_, Twox64Concat, CollectionId, Twox64Concat, NFTId, NFT, ValueQuery>;

	#[derive(Encode, Decode, Default, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	pub struct FundInfo<AccountId, Balance, BlockNumber> {
		/// The account that will recieve the funds if the campaign is successful.
		beneficiary: AccountId,
		/// The amount of deposit placed.
		deposit: Balance,
		/// The total amount raised.
		raised: Balance,
		/// Block number after which funding must have succeeded.
		end: BlockNumber,
	}

	#[pallet::storage]
	#[pallet::getter(fn funds)]
	/// Info on all of the funds.
	pub(super) type Funds<T: Config> =
		StorageMap<_, Blake2_128Concat, FundIndex, FundInfoOf<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn fund_count)]
	/// The total number of funds that have so far been allocated.
	/// Each fund ties to a collection
	pub(super) type FundCount<T: Config> = StorageValue<_, FundIndex, ValueQuery>;

	// Configure the pallet by specifying the parameters and types on which it depends.

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The currency type
		type Currency: ReservableCurrency<Self::AccountId>;
		// type ReservableCurrency: ReservableCurrency<Self::AccountId>;
		type CollectionRandomness: Randomness<Self::Hash, Self::BlockNumber>;
		// The amount to be held on deposit by the owner of a crowdfund.
		type SubmissionDeposit: Get<BalanceOf<Self>>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	impl<T: Config> MaxEncodedLen for CollectionInfo<T> {
		fn max_encoded_len() -> usize {
			let len: usize = 16;
			len
		}
	}

	impl MaxEncodedLen for NFT {
		fn max_encoded_len() -> usize {
			let len: usize = 16;
			len
		}
	}

	impl Default for NFT {
		fn default() -> Self {
			NFT {
				id: 0,
				name: format!("").as_bytes().to_vec().clone(),
				image_url: format!("").as_bytes().to_vec().clone(),
			}
		}
	}

	// // The pallet's runtime storage items.
	// // https://docs.substrate.io/v3/runtime/storage
	// #[pallet::storage]
	// #[pallet::getter(fn projects)]
	// pub type Projects<T: Config> =
	// 	StorageMap<_, Twox64Concat, T::Hash, Project<T>>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		CollectionRegistered(CollectionId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		/// Duplidate collection name
		CollectionExists,
		/// Collection not exists
		CollectionNotExists,

		// Fund index is not existed
		InvalidFundIndex,

		// Require owner to be execute some pallet calls
		NotFundOwner,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn register_collection(
			origin: OriginFor<T>,
			name: Vec<u8>,
			description: Vec<u8>,
			number_of_items: u16,
			mint_fee: BalanceOf<T>,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;

			_ = Self::new_collection(&who, name, description, number_of_items, mint_fee);

			// Self::deposit_event(Event::CollectionRegistered(collection_id.clone()));

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn approve_collection(
			origin: OriginFor<T>,
			collection_id: CollectionId,
			start_date: u32,
			end_date: u32,
		) -> DispatchResult {
			// let who = ensure_root(origin)?;
			let who = ensure_signed(origin)?;

			// Get collection info
			let mut collection =
				Self::get_collections(&collection_id).ok_or(<Error<T>>::CollectionNotExists)?;

			ensure!(who == collection.owner, Error::<T>::NotFundOwner);

			if collection.project_status != ProjectStatus::Approved {
				collection.project_status = ProjectStatus::Approved;
				collection.start_date = Some(start_date);
				collection.end_date = Some(end_date);
				<Collections<T>>::insert(&collection_id, collection);
			}
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn mint(origin: OriginFor<T>, collection_id: CollectionId) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;

			// Get collection info
			let mut collection =
				Self::get_collections(&collection_id).ok_or(<Error<T>>::CollectionNotExists)?;

			if collection.number_of_minted < collection.number_of_items {
				let mint_fee = collection.mint_fee;
				let nft = Self::generate_collection_nft(collection.number_of_minted);
				// Store data on chain
				// let mut nfts =  Self::get_nfts.iter_prefix_values(collection_id)
				let nft_id = nft.id;
				NFTs::<T>::insert(collection_id, nft_id, nft);
				log::info!(
					"A NFT is minted with ID: {:?} in collection id: {:?}",
					nft_id,
					collection_id
				);

				collection.number_of_minted += 1;
				<Collections<T>>::insert(&collection_id, collection);

				let result = Self::contribute(&who.clone(), 0, mint_fee);
				log::info!("contribution result : {:?}", result);
			} else {
			}
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn dispense_fund(origin: OriginFor<T>, index: FundIndex) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let mut fund = Self::funds(index).ok_or(Error::<T>::InvalidFundIndex)?;

			ensure!(who == fund.beneficiary, Error::<T>::NotFundOwner);

			Self::dispense(index);

			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn get_launchpad_collections() -> u32 {
			10
		}

		fn new_collection(
			owner: &T::AccountId,
			name: Vec<u8>,
			description: Vec<u8>,
			number_of_items: u16,
			mint_fee: BalanceOf<T>,
		) -> Result<CollectionId, DispatchError> {
			let collection_id = <FundCount<T>>::get();
			// not protected against overflow, see safemath section
			<FundCount<T>>::put(collection_id + 1);

			let collection_info = CollectionInfo::<T> {
				id: collection_id,
				owner: owner.clone(),
				name,
				description,
				number_of_items,
				project_status: ProjectStatus::Draft,
				is_frozen: false,
				number_of_minted: 0,
				mint_fee,
				start_date: None,
				end_date: None,
			};

			// Check if the collection id does not already exist in our storage map
			ensure!(Self::get_collections(&collection_id) == None, <Error<T>>::CollectionExists);

			log::info!("A collection is created with ID: {:?}", collection_id);

			// Save collection on-chain
			Collections::<T>::insert(collection_id, collection_info);

			// Create Fund
			let current_block_number = <frame_system::Pallet<T>>::block_number();
			Self::create_fund(&owner, collection_id, current_block_number);
			Ok(collection_id)
		}

		fn generate_collection_nft(total_nft: u16) -> NFT {
			let index = Self::gen_nft_index();
			log::info!("A collection NFT Index is created: {:?}", index);

			// Generate 10 NFTs for a collection
			let images: [&str; 10] = [
				"https://gateway.pinata.cloud/ipfs/QmXYEXK4gNtnydgXBBa37YVhF1Zyi3frYMNuLP2YwNZ6GT",
				"https://gateway.pinata.cloud/ipfs/QmTvzJ6bWkB87BtbUMDnuTE2WqBp2FbDXL6epr6ysRPiXm",
				"https://gateway.pinata.cloud/ipfs/Qmddbben3DWctUaT9kqR6zckPj5DGdNcGQoxq3rMvsDYiL",
				"https://gateway.pinata.cloud/ipfs/QmaiKJcgeWgY5XX6D41pcZti8NYuUMYKQrPbqbBNfBvLpH",
				"https://gateway.pinata.cloud/ipfs/QmaVjEKkUJ5UxcecD3dYAGMCfj7whT6wf2ZBCAF24d3hgf",
				"https://gateway.pinata.cloud/ipfs/Qmcu8x8Hsu4ht7jmWe1c3Dh93p4qn97tenR1uFp1KkN1oC",
				"https://gateway.pinata.cloud/ipfs/QmVXYnjPszRpHEDrNFK1vVAqbQioy6fjweCbq8JHrpGs6j",
				"https://gateway.pinata.cloud/ipfs/QmXm5UZivCrSNoB4VLMcwtkyejNcXGTZBtEJXAq8v3ZFFb",
				"https://gateway.pinata.cloud/ipfs/QmYmte4mZEVLNRtmHtixnasLna7jP86ftZzTsSygvLfNBf",
				"https://gateway.pinata.cloud/ipfs/QmR6uKp5gPWkKHEwWdLhbcjgkE1GWBW49AhYi9V8u4Ly5W",
			];

			NFT {
				id: total_nft,
				name: format!("Item #{}", total_nft).as_bytes().to_vec().clone(),
				image_url: images[index as usize].as_bytes().to_vec().clone(),
			}
		}

		// Note the warning above about saturated conversions
		fn u8_to_balance_saturated(input: u8) -> BalanceOf<T> {
			input.into()
		}

		fn gen_nft_index() -> u8 {
			let random = T::CollectionRandomness::random(&b"NFT Indexing"[..]).0;
			random.as_ref()[0] % 10
		}

		pub fn fund_account_id(index: FundIndex) -> T::AccountId {
			PALLET_ID.into_sub_account(index)
		}
	}

	impl<T: Config> Pallet<T> {
		// CrowdFund
		pub fn create_fund(
			owner: &T::AccountId,
			fund_index: FundIndex,
			end: T::BlockNumber,
		) -> DispatchResultWithPostInfo {
			let deposit = T::SubmissionDeposit::get();
			let imb = T::Currency::withdraw(
				&owner,
				deposit,
				WithdrawReasons::TRANSFER,
				ExistenceRequirement::AllowDeath,
			)?;

			// No fees are paid here if we need to create this account; that's why we don't just
			// use the stock `transfer`.
			let fund_account_id = Self::fund_account_id(fund_index);
			let result = T::Currency::resolve_creating(&fund_account_id, imb);

			log::info!(
				"Creating funding pot result: {:?} with account id: {:?}",
				result,
				fund_account_id
			);

			<Funds<T>>::insert(
				fund_index,
				FundInfo { beneficiary: owner.clone(), deposit, raised: Zero::zero(), end },
			);
			log::info!("A fund spot is created: {:?}", fund_index);
			Ok(().into())
		}

		pub fn contribute(
			contributor: &T::AccountId,
			index: FundIndex,
			value: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let mut fund = Self::funds(index).ok_or(Error::<T>::InvalidFundIndex)?;
			let fund_account_id = Self::fund_account_id(index);

			log::info!("DEBUG 1");
			// Add contribution to the fund
			let result = T::Currency::transfer(
				&contributor,
				&fund_account_id,
				value,
				ExistenceRequirement::AllowDeath,
			)?;

			log::info!("Transfer result: {:?}", result);
			fund.raised += value;
			log::info!("A fund spot is contributed: {:?}", value);

			// Check account id total balance
			let current_funding = T::Currency::total_balance(&fund_account_id);
			log::info!("current_funding: {:?}", current_funding);
			Funds::<T>::insert(index, &fund);
			Ok(().into())
		}

		pub fn dispense(index: FundIndex) -> DispatchResultWithPostInfo {
			let fund = Self::funds(index).ok_or(Error::<T>::InvalidFundIndex)?;
			let account = Self::fund_account_id(index);
			let result = T::Currency::resolve_creating(
				&fund.beneficiary,
				T::Currency::withdraw(
					&account,
					fund.raised,
					WithdrawReasons::TRANSFER,
					ExistenceRequirement::AllowDeath,
				)?,
			);
			log::info!("Dispense result: {:?}", result);
			Ok(().into())
		}
	}
}

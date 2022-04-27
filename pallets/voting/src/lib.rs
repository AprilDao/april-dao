#![cfg_attr(not(feature = "std"), no_std)]
// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

	use super::*;
	use codec::{Codec, FullCodec, MaxEncodedLen};
	use frame_support::dispatch::{fmt::Debug, HasCompact};
	use frame_support::pallet_prelude::*;
	use frame_support::sp_runtime::traits::Scale;
	use frame_support::traits::tokens::fungibles::Transfer;
	use frame_support::traits::tokens::nonfungibles::Inspect;
	use frame_support::traits::Time;
	use frame_system::pallet_prelude::*;
	use scale_info::{StaticTypeInfo, TypeInfo};
	use sp_std::vec::Vec;
	use sp_arithmetic::per_things::Percent;

	// Local pallet
	use pallet_collection::FundInfoInterface;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type NFT: Inspect<Self::AccountId, InstanceId = Self::NFTInstance, ClassId = Self::NFTClass>;
		type Asset: Transfer<Self::AccountId>;
		type AssetId: Member
			+ Parameter
			+ Default
			+ Copy
			+ HasCompact
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen
			+ TypeInfo;

		type NFTClass: Member
			+ Parameter
			+ Default
			+ Copy
			+ HasCompact
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen
			+ TypeInfo;
		type NFTInstance: Member
			+ Parameter
			+ Default
			+ Copy
			+ HasCompact
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen
			+ TypeInfo;
		/// The maximum length book string type.
		#[pallet::constant]
		type MaxProposal: Get<u32>;
		#[pallet::constant]
		type MaxVoter: Get<u32>;
		#[pallet::constant]
		type MaxStringLength: Get<u32>;
		#[pallet::constant]
		type AgreementPercenagethresHold: Get<Percent>;
		type ProposalId: FullCodec + Copy + Eq + PartialEq + Debug + TypeInfo + MaxEncodedLen;
		type Balance: Parameter
			+ Member
			+ Codec
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize
			+ Debug
			+ MaxEncodedLen
			+ TypeInfo;
		type Moment: Parameter
			+ Default
			+ Scale<Self::BlockNumber, Output = Self::Moment>
			+ Copy
			+ MaxEncodedLen
			+ StaticTypeInfo
			+ MaybeSerializeDeserialize
			+ Send;
		// + RuntimeDeserialize;
		type Timestamp: Time<Moment = Self::Moment>;

		// pallet-collection loose coupling
		type FundInfoImpl: FundInfoInterface<Self>;
	}

	// Pallets use events to inform users when important changes are made.
	// Event documentation should end with an array that provides descriptive names for parameters.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NFTClassBinded(T::AccountId, T::AssetId, T::NFTClass),
		ProposalCreated(T::AccountId, T::ProposalId),
		Voted(T::AccountId, T::ProposalId, T::NFTClass, T::NFTInstance, bool),
		Excuted(T::AccountId, T::ProposalId, bool),
	}
	#[pallet::error]
	pub enum Error<T> {
		TooLong,
		ProposalAlreadyExists,
		NoneValue,
		VoterIsNotNFTOwner,
		ProposalNotFound,
		NFTNotAvailable,
		NFTIsNotExist,
		NFTAlreadyBindedToAnotherAsset,
		CollectionNotExists,
		ProposalNotExists,
		HaveNotPassTheThresHold,
	}
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn get_proposal_id)]
	pub(super) type ProposalIds<T: Config> =
		StorageValue<_, BoundedVec<T::ProposalId, T::MaxProposal>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_available_voting_nft)]
	pub(super) type AvailableVotingNFT<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AssetId, T::NFTClass, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_proposals)]
	pub(super) type Proposals<T: Config> =
		StorageMap<_, Blake2_128Concat, T::ProposalId, Proposal<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_proposal_collections)]
	pub(super) type ProposalCollections<T: Config> =
		StorageMap<_, Blake2_128Concat, T::ProposalId, u32, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_votes)]
	pub(super) type Votes<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::ProposalId,
		BoundedVec<Vote<T>, T::MaxVoter>,
		OptionQuery,
	>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(50_000_000)]
		pub fn create_proposal(
			origin: OriginFor<T>,
			proposal_id: T::ProposalId,
			collection_id: u32,
			asset_id: T::AssetId,
			amount_withdraw: T::Balance,
			wallet_address: T::AccountId,
			title: Vec<u8>,
			description: Vec<u8>,
			expired_at: T::Moment,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let bounded_description: BoundedVec<u8, T::MaxStringLength> =
				description.try_into().map_err(|()| Error::<T>::TooLong)?;
			let bounded_title: BoundedVec<u8, T::MaxStringLength> =
				title.try_into().map_err(|()| Error::<T>::TooLong)?;

			match <Proposals<T>>::get(&proposal_id) {
				Some(_) => Err(Error::<T>::ProposalAlreadyExists)?,
				None => {},
			}
			let proposal = Proposal::<T> {
				proposer: sender.clone(),
				title: bounded_title,
				description: bounded_description,
				wallet_address,
				asset_id,
				amount_withdraw,
				expired_at,
			};

			<Proposals<T>>::insert(&proposal_id, &proposal);
			<ProposalCollections<T>>::insert(&proposal_id, collection_id);
			match <ProposalIds<T>>::get() {
				Some(proposal_ids) => {
					let mut vec_ids = proposal_ids.to_vec();
					vec_ids.push(proposal_id);
					let proposal_ids: BoundedVec<T::ProposalId, T::MaxProposal> =
						vec_ids.try_into().map_err(|()| Error::<T>::TooLong)?;
					<ProposalIds<T>>::put(proposal_ids);
				},

				None => {
					let val = Some(
						BoundedVec::<T::ProposalId, T::MaxProposal>::try_from(
							Vec::<T::ProposalId>::new(),
						)
						.unwrap(),
					);
					<ProposalIds<T>>::set(val);
				},
			}

			<Votes<T>>::insert(
				&proposal_id,
				&BoundedVec::<Vote<T>, T::MaxVoter>::try_from(Vec::<Vote<T>>::new()).unwrap(),
			);
			Self::deposit_event(Event::ProposalCreated(sender, proposal_id));
			Ok(())
		}

		#[pallet::weight(50_000_000)]
		pub fn vote(
			origin: OriginFor<T>,
			proposal_id: T::ProposalId,
			is_accepted: bool,
			nft_class: T::NFTClass,
			nft_instance: T::NFTInstance,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			if let Some(proposal) = <Proposals<T>>::get(&proposal_id) {
				if sender == T::NFT::owner(&nft_class, &nft_instance).unwrap() {
					let votes = <Votes<T>>::get(&proposal_id).unwrap();
					let mut vec_votes = votes.to_vec();
					let vote = Vote { voter: sender.clone(), nft: nft_instance, is_accepted };
					vec_votes.push(vote);
					let votes: BoundedVec<Vote<T>, T::MaxVoter> =
						vec_votes.try_into().map_err(|()| Error::<T>::TooLong)?;
					<Votes<T>>::insert(&proposal_id, votes);
					Self::deposit_event(Event::Voted(
						sender,
						proposal_id,
						nft_class,
						nft_instance,
						is_accepted,
					));
					Ok(())
				} else {
					Err(Error::<T>::VoterIsNotNFTOwner)?
				}
			} else {
				Err(Error::<T>::ProposalNotFound)?
			}
		}

		#[pallet::weight(50_000_000)]
		pub fn execute(origin: OriginFor<T>, proposal_id: T::ProposalId) -> DispatchResult {
			let collection_id = <ProposalCollections<T>>::get(&proposal_id).ok_or(<Error<T>>::CollectionNotExists)?;
			let proposal = <Proposals<T>>::get(&proposal_id).ok_or(<Error<T>>::ProposalNotExists)?;
			let beneficiary = proposal.wallet_address;
			// Assure the acceptance percentage is greater than the threshold
			let result = Self::assure_proposal_is_accepted(proposal_id);
			if result.is_err() {
				result
			} else {
				let _ = T::FundInfoImpl::dispense(origin, collection_id, beneficiary);
				Ok(())
			}		
		}

		#[pallet::weight(50_000_000)]
		pub fn bind_asset_to_nft(
			origin: OriginFor<T>,
			asset_id: T::AssetId,
			nft_class: T::NFTClass,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			if <AvailableVotingNFT<T>>::get(&asset_id).is_none() {
				<AvailableVotingNFT<T>>::insert(&asset_id, &nft_class);
			} else {
				Err(Error::<T>::NFTAlreadyBindedToAnotherAsset)?
			}
			Self::deposit_event(Event::NFTClassBinded(sender, asset_id, nft_class));
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn assure_proposal_is_accepted(proposal_id: T::ProposalId) -> DispatchResult {
			// Acceptance percentage
			let votes = <Votes<T>>::get(&proposal_id).unwrap();
			let accepted_count = votes.iter().filter(|x| x.is_accepted).count();
			let total_votes = votes.iter().size_hint().0;

			let threshold = T::AgreementPercenagethresHold::get();
			let p = Percent::from_rational(accepted_count, total_votes);
			if p <= threshold {
				Err(Error::<T>::HaveNotPassTheThresHold)?
			} else {
				Ok(())
			}
		}

	}
	#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Proposal<T: Config> {
		pub proposer: T::AccountId,
		pub amount_withdraw: T::Balance,
		pub title: BoundedVec<u8, T::MaxStringLength>,
		pub description: BoundedVec<u8, T::MaxStringLength>,
		pub wallet_address: T::AccountId,
		pub asset_id: T::AssetId,
		pub expired_at: T::Moment,
	}

	impl<T: Config> MaxEncodedLen for Proposal<T> {
		fn max_encoded_len() -> usize {
			BoundedVec::<u8, T::MaxStringLength>::max_encoded_len() * 5
		}
	}

	#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Vote<T: Config> {
		voter: T::AccountId,
		is_accepted: bool,
		nft: T::NFTInstance,
	}

	impl<T: Config> MaxEncodedLen for Vote<T> {
		fn max_encoded_len() -> usize {
			BoundedVec::<u8, T::MaxStringLength>::max_encoded_len() * 3
		}
	}
}

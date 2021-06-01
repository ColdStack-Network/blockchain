#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
    dispatch::DispatchResultWithPostInfo, 
    pallet_prelude::*,
    weights::{Pays},
  };
	use frame_system::pallet_prelude::*;
  use sp_std::vec::Vec;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn key)]
	// Learn more about declaring storage items:
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
  pub type Key<T: Config> = StorageValue<_, T::AccountId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn total_file_count)]
  pub type TotalFileCount<T: Config> = StorageValue<_, u128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn total_file_size)]
  pub type TotalFileSize<T: Config> = StorageValue<_, u128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn total_issuance)]
  pub type TotalIssuance<T: Config> = StorageValue<_, u128, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn locked_funds)]
  pub type LockedFunds<T: Config> = StorageValue<_, u128, ValueQuery>;

	#[pallet::storage]
	pub type Balances<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		Vec<u8>,
		u128,
		ValueQuery
	>;

	#[pallet::storage]
	pub type FilePermissionOwnersByETHAddress<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		Vec<u8>,
		T::AccountId,
		ValueQuery
	>;

	#[pallet::storage]
	pub type FilePermissionOwnersByAccountId<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Vec<u8>,
		ValueQuery
	>;

	#[pallet::storage]
	pub type BillingPermissionOwnersByETHAddress<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		Vec<u8>,
		T::AccountId,
		ValueQuery
	>;

	#[pallet::storage]
	pub type BillingPermissionOwnersByAccountId<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Vec<u8>,
		ValueQuery
	>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub key: T::AccountId,
    pub total_issuance: u128,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				key: Default::default(),
        total_issuance: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			<Key<T>>::put(&self.key);
			<TotalFileCount<T>>::put(0);
			<TotalFileSize<T>>::put(0);
      <TotalIssuance<T>>::put(&self.total_issuance);
      <LockedFunds<T>>::put(&self.total_issuance);
		}
	}

	// Pallets use events to inform users when important changes are made.
	// https://substrate.dev/docs/en/knowledgebase/runtime/events
	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
    // TODO pass custom struct to event?
    Upload(
      /*user_eth_address*/     Vec<u8>,
      /*file_name_hash:*/      Vec<u8>,
      /*file_size_bytes:*/     u128,
      /*file_contents_hash:*/  Vec<u8>,
      /*gateway_eth_address:*/ Vec<u8>,
    ),
    Delete(Vec<u8>,Vec<u8>),
    Deposit(Vec<u8>, u128),
    Withdraw(Vec<u8>, u128),
    FilePermissionGranted(Vec<u8>, T::AccountId),
    FilePermissionRevoked(Vec<u8>, T::AccountId),
    BillingPermissionGranted(Vec<u8>, T::AccountId),
    BillingPermissionRevoked(Vec<u8>, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
    InvalidArguments,
    Unauthorized,
    InsufficientIssuance,
    InsufficientFunds,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}




	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {

		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight((0, Pays::No))]
		pub fn upload(origin: OriginFor<T>, 
      user_eth_address: Vec<u8>,
      file_name_hash: Vec<u8>,
      file_size_bytes: u128,
      file_contents_hash: Vec<u8>,
      gateway_eth_address: Vec<u8>,
    ) -> DispatchResultWithPostInfo {
      let sender = ensure_signed(origin)?;

      let has_permission = 
        // is admin
        sender == Self::key()
        ||
        FilePermissionOwnersByAccountId::<T>::contains_key(&sender);

      ensure!(has_permission, Error::<T>::Unauthorized);

      ensure!(user_eth_address.len() == 20, Error::<T>::InvalidArguments);
      ensure!(gateway_eth_address.len() == 20, Error::<T>::InvalidArguments);
      ensure!(file_contents_hash.len() == 32, Error::<T>::InvalidArguments);
      ensure!(file_name_hash.len() == 32, Error::<T>::InvalidArguments);

      <TotalFileCount<T>>::put(Self::total_file_count() + 1);
      <TotalFileSize<T>>::put(Self::total_file_size() + file_size_bytes);

      Self::deposit_event(Event::Upload(
        user_eth_address, 
        file_name_hash, 
        file_size_bytes,
        file_contents_hash, 
        gateway_eth_address,
      ));

      Ok(().into())
		}

		#[pallet::weight((0, Pays::No))]
    fn delete(origin: OriginFor<T>, 
      user_eth_address: Vec<u8>, 
      file_name_hash: Vec<u8>,
    ) -> DispatchResultWithPostInfo {
      let sender = ensure_signed(origin)?;

      let has_permission = 
        // is admin
        sender == Self::key()
        ||
        FilePermissionOwnersByAccountId::<T>::contains_key(&sender);
      ensure!(has_permission, Error::<T>::Unauthorized);

      ensure!(user_eth_address.len() == 20, Error::<T>::InvalidArguments);
      ensure!(file_name_hash.len() == 32, Error::<T>::InvalidArguments);

      // TODO total_file_count, total_file_size

      Self::deposit_event(Event::Delete(user_eth_address, file_name_hash));
      Ok(().into())
    }

		#[pallet::weight((0, Pays::No))]
    fn deposit(origin: OriginFor<T>,
      account: Vec<u8>, value: u128
    ) -> DispatchResultWithPostInfo {
      let sender = ensure_signed(origin)?;

      let has_permission = 
        // is admin
        sender == Self::key()
        ||
        BillingPermissionOwnersByAccountId::<T>::contains_key(&sender);

      ensure!(has_permission, Error::<T>::Unauthorized);

      ensure!(account.len() == 20, Error::<T>::InvalidArguments);

      let locked_funds = LockedFunds::<T>::get();

      ensure!(locked_funds >= value, Error::<T>::InsufficientIssuance);

      if !Balances::<T>::contains_key(&account) {
        Balances::<T>::insert(&account, value);
      } else {
        let current_balance = Balances::<T>::get(&account);
        // TODO overflow?
        Balances::<T>::insert(&account, current_balance + value);
      }
      // TODO comment why it cannot overflow
      LockedFunds::<T>::put(locked_funds - value);
      Self::deposit_event(Event::Deposit(account, value));
      Ok(().into())
    }

		#[pallet::weight((0, Pays::No))]
    fn withdraw(origin: OriginFor<T>,
      account: Vec<u8>, value: u128
    ) -> DispatchResultWithPostInfo {
      let sender = ensure_signed(origin)?;

      let has_permission = 
        // is admin
        sender == Self::key()
        ||
        BillingPermissionOwnersByAccountId::<T>::contains_key(&sender);

      ensure!(has_permission, Error::<T>::Unauthorized);

      ensure!(account.len() == 20, Error::<T>::InvalidArguments);

      ensure!(Balances::<T>::contains_key(&account), Error::<T>::InsufficientFunds);
      let balance = Balances::<T>::get(&account);
      ensure!(balance >= value, Error::<T>::InsufficientFunds);
      let next_balance = balance - value;
      Balances::<T>::insert(&account, next_balance);
      // TODO comment why it can never overflow
      LockedFunds::<T>::put(LockedFunds::<T>::get() + value);
      Ok(().into())
    }

		#[pallet::weight((0, Pays::No))]
    fn grant_file_permission(origin: OriginFor<T>,
      eth_address: Vec<u8>, 
      account_id: T::AccountId,
    ) -> DispatchResultWithPostInfo {
      let sender = ensure_signed(origin)?;
      ensure!(sender == Self::key(), Error::<T>::Unauthorized);

      ensure!(eth_address.len() == 20, Error::<T>::InvalidArguments);

      if !FilePermissionOwnersByETHAddress::<T>::contains_key(&eth_address) {
        let current_account_id = FilePermissionOwnersByETHAddress::<T>::get(&eth_address);
        FilePermissionOwnersByAccountId::<T>::take(&current_account_id);
      }
      FilePermissionOwnersByETHAddress::<T>::insert(&eth_address, &account_id);
      FilePermissionOwnersByAccountId::<T>::insert(&account_id, &eth_address);
      Self::deposit_event(Event::FilePermissionGranted(eth_address, account_id));
      Ok(().into())
    }

		#[pallet::weight((0, Pays::No))]
    fn grant_billing_permission(origin: OriginFor<T>,
      eth_address: Vec<u8>, 
      account_id: T::AccountId,
    ) -> DispatchResultWithPostInfo {
      let sender = ensure_signed(origin)?;
      ensure!(sender == Self::key(), Error::<T>::Unauthorized);

      ensure!(eth_address.len() == 20, Error::<T>::InvalidArguments);

      if !BillingPermissionOwnersByETHAddress::<T>::contains_key(&eth_address) {
        let current_account_id = BillingPermissionOwnersByETHAddress::<T>::get(&eth_address);
        BillingPermissionOwnersByAccountId::<T>::take(&current_account_id);
      }
      BillingPermissionOwnersByETHAddress::<T>::insert(&eth_address, &account_id);
      BillingPermissionOwnersByAccountId::<T>::insert(&account_id, &eth_address);
      Self::deposit_event(Event::BillingPermissionGranted(eth_address, account_id));
      Ok(().into())
    }

		#[pallet::weight((0, Pays::No))]
    fn revoke_file_permission(origin: OriginFor<T>,
      eth_address: Vec<u8>, 
    ) -> DispatchResultWithPostInfo {
      let sender = ensure_signed(origin)?;
      ensure!(sender == Self::key(), Error::<T>::Unauthorized);
      ensure!(eth_address.len() == 20, Error::<T>::InvalidArguments);
      ensure!(FilePermissionOwnersByETHAddress::<T>::contains_key(&eth_address), 
                                        Error::<T>::InvalidArguments);
      let account_id = FilePermissionOwnersByETHAddress::<T>::take(&eth_address);
      FilePermissionOwnersByAccountId::<T>::take(&account_id);
      Self::deposit_event(Event::FilePermissionRevoked(eth_address, account_id));
      Ok(().into())
    }

		#[pallet::weight((0, Pays::No))]
    fn revoke_billing_permission(origin: OriginFor<T>,
      eth_address: Vec<u8>, 
    ) -> DispatchResultWithPostInfo {
      let sender = ensure_signed(origin)?;
      ensure!(sender == Self::key(), Error::<T>::Unauthorized);
      ensure!(eth_address.len() == 20, Error::<T>::InvalidArguments);
      ensure!(BillingPermissionOwnersByETHAddress::<T>::contains_key(&eth_address), 
                                        Error::<T>::InvalidArguments);
      let account_id = BillingPermissionOwnersByETHAddress::<T>::take(&eth_address);
      BillingPermissionOwnersByAccountId::<T>::take(&account_id);
      Self::deposit_event(Event::BillingPermissionRevoked(eth_address, account_id));
      Ok(().into())
    }

	}
}

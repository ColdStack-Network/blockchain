#![cfg_attr(not(feature = "std"), no_std)]

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
  use codec::{Encode, Decode};
  use sp_std::vec::Vec;

  #[pallet::config]
  pub trait Config: frame_system::Config {
    type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
  }

  #[pallet::pallet]
  #[pallet::generate_store(pub(super) trait Store)]
  pub struct Pallet<T>(_);

  #[pallet::storage]
  #[pallet::getter(fn key)]

  // Admin account key
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

  /*
    Map node eth address -> node url
  */
  #[pallet::storage]
  pub type NodeURLs<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    Vec<u8>,
    Vec<u8>,
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

  #[derive(Default, Clone, Debug, PartialEq, Encode, Decode)]
  pub struct Gateway {
    address: Vec<u8>,
    seed_address: Option<Vec<u8>>,
    storage: u8,
  }

  /*
    Map gateway node address -> gateway
  */
  #[pallet::storage]
  pub type Gateways<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    Vec<u8>,
    Gateway,
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

  #[pallet::event]
  #[pallet::metadata(T::AccountId = "AccountId")]
  #[pallet::generate_deposit(pub(super) fn deposit_event)]
  pub enum Event<T: Config> {
    Upload(
      /*user_eth_address*/     Vec<u8>,
      /*file_name_hash:*/      Vec<u8>,
      /*file_size_bytes:*/     u128,
      /*file_contents_hash:*/  Vec<u8>,
      /*gateway_eth_address:*/ Vec<u8>,
      /*filenode_eth_address:*/Vec<u8>,
    ),
    Download(
      /*user_eth_address*/     Vec<u8>,
      /*file_name_hash:*/      Vec<u8>,
      /*file_size_bytes:*/     u128,
      /*file_contents_hash:*/  Vec<u8>,
      /*gateway_eth_address:*/ Vec<u8>,
      /*filenode_eth_address:*/Vec<u8>,
    ),
    Delete(
      Vec<u8>, /* user_eth_address */
      Vec<u8>, /* file_name_hash */
      Vec<u8>, /* filenode_eth_address */
    ),
    Deposit(Vec<u8>, u128),
    Withdraw(Vec<u8>, u128),
    Transfer(Vec<u8>, Vec<u8>, u128),
    FilePermissionGranted(Vec<u8>, T::AccountId, Vec<u8>),
    FilePermissionRevoked(Vec<u8>, T::AccountId),
    BillingPermissionGranted(Vec<u8>, T::AccountId, Vec<u8>),
    BillingPermissionRevoked(Vec<u8>, T::AccountId),
    GatewayNodeRegistered(Vec<u8>, Option<Vec<u8>>, u8, Vec<u8>),
  }

  #[pallet::error]
  pub enum Error<T> {
    InvalidArguments,
    Unauthorized,
    InsufficientIssuance,
    InsufficientFunds,
  }

  #[pallet::hooks]
  impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

  #[pallet::call]
  impl<T: Config> Pallet<T> {

    #[pallet::weight((0, Pays::No))]
    pub fn upload(origin: OriginFor<T>,
      user_eth_address: Vec<u8>,
      file_name_hash: Vec<u8>,
			file_storage_class: u8,
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

      let filenode_eth_address = FilePermissionOwnersByAccountId::<T>::get(&sender);
			let file_storage_class = 0;
			
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
        filenode_eth_address,
        file_storage_class
      ));

      Ok(().into())
    }

    #[pallet::weight((0, Pays::No))]
    pub fn download(origin: OriginFor<T>, 
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

      let filenode_eth_address = FilePermissionOwnersByAccountId::<T>::get(&sender);

      ensure!(has_permission, Error::<T>::Unauthorized);

      ensure!(user_eth_address.len() == 20, Error::<T>::InvalidArguments);
      ensure!(gateway_eth_address.len() == 20, Error::<T>::InvalidArguments);
      ensure!(file_contents_hash.len() == 32, Error::<T>::InvalidArguments);
      ensure!(file_name_hash.len() == 32, Error::<T>::InvalidArguments);

      Self::deposit_event(Event::Download(
        user_eth_address, 
        file_name_hash, 
        file_size_bytes,
        file_contents_hash, 
        gateway_eth_address,
        filenode_eth_address,
      ));

      Ok(().into())
    }

    #[pallet::weight((0, Pays::No))]
    pub fn delete(origin: OriginFor<T>, 
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

      let filenode_eth_address = FilePermissionOwnersByAccountId::<T>::get(&sender);

      ensure!(user_eth_address.len() == 20, Error::<T>::InvalidArguments);
      ensure!(file_name_hash.len() == 32, Error::<T>::InvalidArguments);

      // TODO total_file_count, total_file_size

      Self::deposit_event(Event::Delete(
        user_eth_address, 
        file_name_hash, 
        filenode_eth_address
      ));
      Ok(().into())
    }

    #[pallet::weight((0, Pays::No))]
    pub fn deposit(origin: OriginFor<T>,
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
        // `current_balance + value` cannot overflow because it cannot be
        // greater than total_issuance
        Balances::<T>::insert(&account, current_balance + value);
      }
      // `locked_funds` is more than `value` (see check earlier)
      LockedFunds::<T>::put(locked_funds - value);
      Self::deposit_event(Event::Deposit(account, value));
      Ok(().into())
    }

    #[pallet::weight((0, Pays::No))]
    pub fn withdraw(origin: OriginFor<T>,
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
      // `locked_funds + value` cannot overflow because it cannot be greater
      // than total_issuance
      LockedFunds::<T>::put(LockedFunds::<T>::get() + value);
      Self::deposit_event(Event::Withdraw(account, value));
      Ok(().into())
    }

    #[pallet::weight((0, Pays::No))]
    pub fn transfer(origin: OriginFor<T>,
      from: Vec<u8>, 
      to: Vec<u8>, 
      value: u128,
    ) -> DispatchResultWithPostInfo {
      let sender = ensure_signed(origin)?;

      let has_permission = 
        // is admin
        sender == Self::key()
        ||
        BillingPermissionOwnersByAccountId::<T>::contains_key(&sender);

      ensure!(has_permission, Error::<T>::Unauthorized);

      ensure!(from.len() == 20, Error::<T>::InvalidArguments);
      ensure!(to.len() == 20, Error::<T>::InvalidArguments);

      ensure!(Balances::<T>::contains_key(&from), Error::<T>::InsufficientFunds);
      let balance = Balances::<T>::get(&from);
      ensure!(balance >= value, Error::<T>::InsufficientFunds);
      Balances::<T>::insert(&from, balance - value);

      if !Balances::<T>::contains_key(&to) {
        Balances::<T>::insert(&to, value);
      } else {
        let balance = Balances::<T>::get(&to);
        // `balance + value` cannot overflow because it always less than
        // total_issuance
        Balances::<T>::insert(&to, balance + value);
      }

      Self::deposit_event(Event::Transfer(from, to, value));
      Ok(().into())
    }

    #[pallet::weight((0, Pays::No))]
    pub fn grant_file_permission(origin: OriginFor<T>,
      eth_address: Vec<u8>, 
      account_id: T::AccountId,
      node_url: Vec<u8>,
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
      NodeURLs::<T>::insert(&eth_address, &node_url);
      Self::deposit_event(Event::FilePermissionGranted(eth_address, account_id, node_url));
      Ok(().into())
    }

    #[pallet::weight((0, Pays::No))]
    pub fn grant_billing_permission(origin: OriginFor<T>,
      eth_address: Vec<u8>, 
      account_id: T::AccountId,
      node_url: Vec<u8>,
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
      NodeURLs::<T>::insert(&eth_address, &node_url);
      Self::deposit_event(Event::BillingPermissionGranted(eth_address, account_id, node_url));
      Ok(().into())
    }

    #[pallet::weight((0, Pays::No))]
    pub fn revoke_file_permission(origin: OriginFor<T>,
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
    pub fn revoke_billing_permission(origin: OriginFor<T>,
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

    #[pallet::weight((0, Pays::No))]
    pub fn register_gateway_node(origin: OriginFor<T>,
      eth_address: Vec<u8>, 
      seed_eth_address: Option<Vec<u8>>,
      storage: u8,
      node_url: Vec<u8>,
    ) -> DispatchResultWithPostInfo {
      let sender = ensure_signed(origin)?;
      ensure!(sender == Self::key(), Error::<T>::Unauthorized);
      ensure!(eth_address.len() == 20, Error::<T>::InvalidArguments);
      if let Some(ref addr) = seed_eth_address {
        ensure!(addr.len() == 20, Error::<T>::InvalidArguments);
      }
      let gateway = Gateway {
        address: eth_address.clone(),
        seed_address: seed_eth_address.clone(),
        storage: storage,
      };
      Gateways::<T>::insert(eth_address.clone(), &gateway);
      NodeURLs::<T>::insert(eth_address.clone(), &node_url);
      Self::deposit_event(Event::GatewayNodeRegistered(
        eth_address,
        seed_eth_address,
        storage,
        node_url,
      ));
      Ok(().into())
    }

  }
}

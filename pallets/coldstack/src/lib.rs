#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_module, decl_storage, decl_event, decl_error, ensure,
    weights::{Pays, DispatchClass},
};


use frame_system::ensure_signed;
use sp_std::vec::Vec;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Config: frame_system::Config {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}


// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
    trait Store for Module<T: Config> as ColdStack {
        // Admin account key
        Key get(fn key) config(): T::AccountId;
        Balances: map hasher(blake2_128_concat) Vec<u8> => u128;

        // Accounts that have permission to add files
        FilePermissionOwners: map hasher(blake2_128_concat) T::AccountId => ();

        // Accounts that have permission to add files
        BillingPermissionOwners: map hasher(blake2_128_concat) T::AccountId => ();
    }
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        // Errors must be initialized if they are used by the pallet.
        type Error = Error<T>;

        // Events must be initialized if they are used by the pallet.
        fn deposit_event() = default;

        #[weight = (10_000, DispatchClass::Normal, Pays::No)]
        fn upload(origin, 
          bucket_name_hash: Vec<u8>,
          file_contents_hash: Vec<u8>,
          file_name_hash: Vec<u8>,
          file_size_bytes: u128,
          gateway_eth_address: Vec<u8>,
        ){
          let sender = ensure_signed(origin)?;

          let has_permission = 
            // is admin
            sender == Self::key()
            ||
            FilePermissionOwners::<T>::contains_key(&sender);

          ensure!(has_permission, Error::<T>::Unauthorized);

          ensure!(gateway_eth_address.len() == 20, Error::<T>::InvalidArguments);

          /* We expect hash to be at least16 bytes long here */
          ensure!(bucket_name_hash.len() >= 16, Error::<T>::InvalidArguments);
          ensure!(file_contents_hash.len() >= 16, Error::<T>::InvalidArguments);
          ensure!(file_name_hash.len() >= 16, Error::<T>::InvalidArguments);

          Self::deposit_event(RawEvent::Upload(
            bucket_name_hash, 
            file_contents_hash, 
            file_name_hash, 
            file_size_bytes,
            gateway_eth_address,
          ));
        }

        #[weight = (10_000, DispatchClass::Normal, Pays::No)]
        fn delete(origin, name_hash: Vec<u8>) {
          ensure_signed(origin)?;

          Self::deposit_event(RawEvent::Delete(name_hash));
        }

        #[weight = (10_000, DispatchClass::Normal, Pays::No)]
        fn deposit(origin, account: Vec<u8>, value: u128){
          let sender = ensure_signed(origin)?;

          let has_permission = 
            // is admin
            sender == Self::key()
            ||
            BillingPermissionOwners::<T>::contains_key(&sender);

          ensure!(has_permission, Error::<T>::Unauthorized);

          ensure!(account.len() == 20, Error::<T>::InvalidArguments);

          if !Balances::contains_key(&account) {
            Balances::insert(&account, value);
            Self::deposit_event(RawEvent::Deposit(account, value));
          } else {
            let current_balance = Balances::get(&account);
            // TODO overflow?
            Balances::insert(&account, current_balance + value);
            Self::deposit_event(RawEvent::Deposit(account, value));
          }
        }

        #[weight = (10_000, DispatchClass::Normal, Pays::No)]
        fn withdraw(origin, account: Vec<u8>, value: u128){
          let sender = ensure_signed(origin)?;

          let has_permission = 
            // is admin
            sender == Self::key()
            ||
            BillingPermissionOwners::<T>::contains_key(&sender);

          ensure!(has_permission, Error::<T>::Unauthorized);

          ensure!(account.len() == 20, Error::<T>::InvalidArguments);

          ensure!(Balances::contains_key(&account), Error::<T>::InsufficientFunds);
          let balance = Balances::get(&account);
          ensure!(balance >= value, Error::<T>::InsufficientFunds);
          let next_balance = balance - value;
          Balances::insert(&account, next_balance);
        }

        #[weight = (10_000, DispatchClass::Normal, Pays::No)]
        fn grant_file_permission(origin, account_id: T::AccountId){
          let sender = ensure_signed(origin)?;
          ensure!(sender == Self::key(), Error::<T>::Unauthorized);
          FilePermissionOwners::<T>::insert(&account_id, ());
          Self::deposit_event(RawEvent::FilePermissionGranted(account_id));
        }

        #[weight = (10_000, DispatchClass::Normal, Pays::No)]
        fn grant_billing_permission(origin, account_id: T::AccountId){
          let sender = ensure_signed(origin)?;
          ensure!(sender == Self::key(), Error::<T>::Unauthorized);
          BillingPermissionOwners::<T>::insert(&account_id, ());
          Self::deposit_event(RawEvent::BillingPermissionGranted(account_id));
        }

        #[weight = (10_000, DispatchClass::Normal, Pays::No)]
        fn revoke_file_permission(origin, account_id: T::AccountId){
          let sender = ensure_signed(origin)?;
          ensure!(sender == Self::key(), Error::<T>::Unauthorized);
          ensure!(FilePermissionOwners::<T>::contains_key(&account_id), 
                                            Error::<T>::InvalidArguments);
          FilePermissionOwners::<T>::take(&account_id);
          Self::deposit_event(RawEvent::FilePermissionRevoked(account_id));
        }

        #[weight = (10_000, DispatchClass::Normal, Pays::No)]
        fn revoke_billing_permission(origin, account_id: T::AccountId){
          let sender = ensure_signed(origin)?;
          ensure!(sender == Self::key(), Error::<T>::Unauthorized);
          ensure!(BillingPermissionOwners::<T>::contains_key(&account_id), 
                                            Error::<T>::InvalidArguments);
          BillingPermissionOwners::<T>::take(&account_id);
          Self::deposit_event(RawEvent::BillingPermissionRevoked(account_id));
        }
    }
}

// Pallets use events to inform users when important changes are made.
// Event documentation should end with an array that provides descriptive names for parameters.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event! {
    pub enum Event<T> where AccountId = <T as frame_system::Config>::AccountId {
        // TODO pass custom struct to event?
        Upload(
          /*bucket_name_hash:*/    Vec<u8>,
          /*file_contents_hash:*/  Vec<u8>,
          /*file_name_hash:*/      Vec<u8>,
          /*file_size_bytes:*/     u128,
          /*gateway_eth_address:*/ Vec<u8>,
        ),
        Delete(Vec<u8>,),
        Deposit(Vec<u8>, u128),
        Withdraw(Vec<u8>, u128),
        FilePermissionGranted(AccountId),
        FilePermissionRevoked(AccountId),
        BillingPermissionGranted(AccountId),
        BillingPermissionRevoked(AccountId),
    }
}


// Errors inform users that something went wrong.
decl_error! {
    pub enum Error for Module<T: Config> {
        InvalidArguments,
        Unauthorized,
        InsufficientFunds,
    }
}

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_module, decl_storage, decl_event, decl_error, ensure, StorageMap
};
use frame_system::ensure_signed;
use sp_std::vec::Vec;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event/*<Self>*/> + Into<<Self as frame_system::Trait>::Event>;
}


// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
    trait Store for Module<T: Trait> as ColdStack {
        Key get(fn key) config(): T::AccountId;
    }
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Errors must be initialized if they are used by the pallet.
        type Error = Error<T>;

        // Events must be initialized if they are used by the pallet.
        fn deposit_event() = default;

        #[weight = 10_000]
        // TODO u128 for size?
        fn pay_for_upload(origin, id: Vec<u8>, owner: Vec<u8>, name: Vec<u8>, size_bytes: u128){
          let sender = ensure_signed(origin)?;
          ensure!(sender == Self::key(), Error::<T>::Unauthorized);
          // TODO verify acc length
          // TODO: check args
          Self::deposit_event(/*Raw*/Event::PayForUpload(id, owner, name, size_bytes));
        }
    }
}

// Pallets use events to inform users when important changes are made.
// Event documentation should end with an array that provides descriptive names for parameters.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event! {
    pub enum Event/*<T> where AccountId = <T as frame_system::Trait>::AccountId*/ {
        PayForUpload(
          Vec<u8>, /* file id */
          Vec<u8>, /* owner */
          Vec<u8>, /* filename */
          u128,    /* size bytes */
        ),
    }
}


// Errors inform users that something went wrong.
decl_error! {
    pub enum Error for Module<T: Trait> {
        Unauthorized
    }
}

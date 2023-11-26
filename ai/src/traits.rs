pub(crate) mod mdp;
pub(crate) mod pomdp;


pub trait KeyableState<Key> {
    fn key(&self) -> Key;
}
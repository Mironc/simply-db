use std::{borrow::Borrow, iter::Map};

/// `VecMap` is analog for `HashMap`, but implemented with `Vec`.
///
/// Preserves the order of insertions.
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct VecMap<K, V>
where
    K: Eq,
{
    map: Vec<(K, V)>,
}
impl<K, V> VecMap<K, V>
where
    K: Eq,
{
    /// Creates new empty instance of `VecMap`
    pub fn new() -> Self {
        Self::default()
    }
    /// Returns the number of elements in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use structures::VecMap;
    ///
    /// let mut a = VecMap::new();
    /// assert_eq!(a.len(), 0);
    /// a.insert(1, "a");
    /// assert_eq!(a.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.map.len()
    }
    /// Returns `true` if the map contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use structures::VecMap;
    ///
    /// let mut a = VecMap::new();
    /// assert!(a.is_empty());
    /// a.insert(1, "a");
    /// assert!(!a.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.map.len() == 0
    }

    /// Returns index to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type,
    /// but [`Eq`] on the borrowed form *must* match those for the key type.
    ///
    /// # Examples
    ///
    /// ```
    /// use structures::VecMap;
    ///
    /// let mut map = VecMap::new();
    /// map.insert(1, "a");
    /// map.insert(2, "b");
    /// assert_eq!(map.get_index(&1), Some(0));
    /// assert_eq!(map.get_index(&2), Some(1));
    /// assert_eq!(map.get_index(&3), None);
    /// ```
    pub fn get_index<Q>(&self, key: &Q) -> Option<usize>
    where
        K: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        self.map
            .iter()
            .enumerate()
            .position(|x| x.1.0.borrow() == key)
    }
    /// Returns a reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type,
    /// but [`Eq`] on the borrowed form *must* match those for the key type.
    ///
    /// # Examples
    ///
    /// ```
    /// use structures::VecMap;
    ///
    /// let mut map = VecMap::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.get(&1), Some(&"a"));
    /// assert_eq!(map.get(&2), None);
    /// ```
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        self.map.iter().find_map(|x| {
            if x.0.borrow() == key {
                Some(&x.1)
            } else {
                None
            }
        })
    }
    /// Returns a mutable reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type,
    /// but and [`Eq`] on the borrowed form *must* match those for the key type.
    ///
    /// # Examples
    ///
    /// ```
    /// use structures::VecMap;
    ///
    /// let mut map = VecMap::new();
    /// map.insert(1, "a");
    /// if let Some(x) = map.get_mut(&1) {
    ///     *x = "b";
    /// }
    /// assert_eq!(map[&1], "b");
    /// ```
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        self.map.iter_mut().find_map(|x| {
            if x.0.borrow() == key {
                Some(&mut x.1)
            } else {
                None
            }
        })
    }
    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, [`None`] is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old
    /// value is returned. The key is not updated, though; this matters for
    /// types that can be `==` without being identical.
    ///
    /// # Examples
    ///
    /// ```
    /// use structures::VecMap;
    ///
    /// let mut map = VecMap::new();
    /// assert_eq!(map.insert(37, "a"), None);
    /// assert_eq!(map.is_empty(), false);
    ///
    /// map.insert(37, "b");
    /// assert_eq!(map.insert(37, "c"), Some("b"));
    /// assert_eq!(map[&37], "c");
    /// ```
    pub fn insert(&mut self, key: K, mut value: V) -> Option<V> {
        if let Some(prev_value) = self.get_mut(&key) {
            std::mem::swap(prev_value, &mut value);
            Some(value)
        } else {
            self.map.push((key, value));
            None
        }
    }
    /// Returns `true` if the map contains a value for the specified key.
    ///
    /// The key may be any borrowed form of the map's key type,
    /// but [`Eq`] on the borrowed form *must* match those for the key type.
    ///
    /// # Examples
    ///
    /// ```
    /// use structures::VecMap;
    ///
    /// let mut map = VecMap::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.contains_key(&1), true);
    /// assert_eq!(map.contains_key(&2), false);
    /// ```
    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where
        Q: Eq,
        K: Borrow<Q>,
    {
        self.map.iter().any(|x| x.0.borrow() == key)
    }
    /// An iterator visiting all key-value pairs in arbitrary order.
    /// The iterator element type is `(&'a K, &'a V)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use structures::VecMap;
    ///
    /// let map = VecMap::from([
    ///     ("a", 1),
    ///     ("b", 2),
    ///     ("c", 3),
    /// ]);
    ///
    /// for (key, val) in map.iter() {
    ///     println!("key: {key} val: {val}");
    /// }
    /// ```
    pub fn iter(&self) -> std::slice::Iter<'_, (K, V)> {
        self.map.iter()
    }
    /// An iterator visiting all key-value pairs in arbitrary order,
    /// with mutable references to the values.
    /// The iterator element type is `(&'a K, &'a mut V)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use structures::VecMap;
    ///
    /// let mut map = VecMap::from([
    ///     ("a", 1),
    ///     ("b", 2),
    ///     ("c", 3),
    /// ]);
    ///
    /// // Update all values
    /// for (_, val) in map.iter_mut() {
    ///     *val *= 2;
    /// }
    ///
    /// for (key, val) in &map {
    ///     println!("key: {key} val: {val}");
    /// }
    /// ```
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, (K, V)> {
        self.map.iter_mut()
    }
    /// An iterator visiting all keys in arbitrary order.
    /// The iterator element type is `&'a K`.
    ///
    /// # Examples
    ///
    /// ```
    /// use structures::VecMap;
    ///
    /// let map = VecMap::from([
    ///     ("a", 1),
    ///     ("b", 2),
    ///     ("c", 3),
    /// ]);
    ///
    /// for key in map.keys() {
    ///     println!("{key}");
    /// }
    /// ```
    pub fn keys(&self) -> Keys<'_, K, V> {
        self.map.iter().map(|x| &x.0)
    }
    /// Creates a consuming iterator visiting all the keys in arbitrary order.
    /// The map cannot be used after calling this.
    /// The iterator element type is `K`.
    ///
    /// # Examples
    ///
    /// ```
    /// use structures::VecMap;
    ///
    /// let map = VecMap::from([
    ///     ("a", 1),
    ///     ("b", 2),
    ///     ("c", 3),
    /// ]);
    ///
    /// let mut vec: Vec<&str> = map.into_keys().collect();
    /// //
    /// assert_eq!(vec, ["a", "b", "c"]);
    /// ```
    pub fn into_keys(self) -> IntoKeys<K, V> {
        self.map.into_iter().map(|x| x.0)
    }
    /// An iterator visiting all values in arbitrary order.
    /// The iterator element type is `&'a V`.
    ///
    /// # Examples
    ///
    /// ```
    /// use structures::VecMap;
    ///
    /// let map = VecMap::from([
    ///     ("a", 1),
    ///     ("b", 2),
    ///     ("c", 3),
    /// ]);
    /// // No need to sort it, because it's in order of insert
    /// for val in map.values() {
    ///     println!("{val}");
    /// }
    /// ```
    pub fn values(&self) -> Values<'_, K, V> {
        self.map.iter().map(|x| &x.1)
    }
    /// Creates a consuming iterator visiting all the values in arbitrary order.
    /// The map cannot be used after calling this.
    /// The iterator element type is `V`.
    ///
    /// # Examples
    ///
    /// ```
    /// use structures::VecMap;
    ///
    /// let map = VecMap::from([
    ///     ("a", 1),
    ///     ("b", 2),
    ///     ("c", 3),
    /// ]);
    ///
    /// let mut vec: Vec<i32> = map.into_values().collect();
    /// // No need to sort it, because it's in order of insert
    /// assert_eq!(vec, [1, 2, 3]);
    /// ```
    pub fn into_values(self) -> IntoValues<K, V> {
        self.map.into_iter().map(|x| x.1)
    }

    pub fn map(&self) -> &[(K, V)] {
        &self.map
    }
}
pub type Keys<'a, K, V> = Map<std::slice::Iter<'a, (K, V)>, fn(&(K, V)) -> &K>;
pub type IntoKeys<K, V> = Map<std::vec::IntoIter<(K, V)>, fn((K, V)) -> K>;
pub type Values<'a, K, V> = Map<std::slice::Iter<'a, (K, V)>, fn(&(K, V)) -> &V>;
pub type IntoValues<K, V> = Map<std::vec::IntoIter<(K, V)>, fn((K, V)) -> V>;
impl<K, V> std::iter::IntoIterator for VecMap<K, V>
where
    K: Eq,
{
    type Item = (K, V);

    type IntoIter = std::vec::IntoIter<(K, V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
    }
}
impl<'a, K, V> std::iter::IntoIterator for &'a VecMap<K, V>
where
    K: Eq,
{
    type Item = &'a (K, V);

    type IntoIter = std::slice::Iter<'a, (K, V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.iter()
    }
}
impl<'a, K, V> std::iter::IntoIterator for &'a mut VecMap<K, V>
where
    K: Eq,
{
    type Item = &'a mut (K, V);

    type IntoIter = std::slice::IterMut<'a, (K, V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.iter_mut()
    }
}
impl<K, Q: ?Sized, V> std::ops::Index<&Q> for VecMap<K, V>
where
    K: Eq + Borrow<Q>,
    Q: Eq,
{
    type Output = V;

    /// Returns a reference to the value corresponding to the supplied key.
    ///
    /// # Panics
    ///
    /// Panics if the key is not present in the `HashMap`.
    #[inline]
    fn index(&self, key: &Q) -> &V {
        self.get(key).expect("no entry found for key")
    }
}
impl<K, V, const N: usize> From<[(K, V); N]> for VecMap<K, V>
where
    K: Eq,
{
    /// Converts a `[(K, V); N]` into a `VecMap<K, V>`.
    ///
    /// If any entries in the array have equal keys,
    /// all but one of the corresponding values will be dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use structures::VecMap;
    ///
    /// let map1 = VecMap::from([(1, 2), (3, 4)]);
    /// let map2: VecMap<_, _> = [(1, 2), (3, 4)].into();
    /// assert_eq!(map1, map2);
    /// ```
    fn from(arr: [(K, V); N]) -> Self {
        Self::from_iter(arr)
    }
}

impl<K, V> From<Vec<(K, V)>> for VecMap<K, V>
where
    K: Eq,
{
    /// Converts a `Vec<(K, V)>` into a `VecMap<K, V>`.
    ///
    /// If any entries in the array have equal keys,
    /// all but one of the corresponding values will be dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use structures::VecMap;
    ///
    /// let map1 = VecMap::from(vec![(1, 2), (3, 4)]);
    /// let map2: VecMap<_, _> = vec![(1, 2), (3, 4)].into();
    /// assert_eq!(map1, map2);
    /// ```
    fn from(vec: Vec<(K, V)>) -> Self {
        Self::from_iter(vec)
    }
}
impl<K, V> Default for VecMap<K, V>
where
    K: Eq,
{
    fn default() -> Self {
        Self {
            map: Default::default(),
        }
    }
}
impl<K, V> FromIterator<(K, V)> for VecMap<K, V>
where
    K: Eq,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> VecMap<K, V> {
        let mut map = VecMap::new();
        map.extend(iter);
        map
    }
}
impl<K, V> Extend<(K, V)> for VecMap<K, V>
where
    K: Eq,
{
    #[inline]
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        self.map.extend(iter)
    }
}

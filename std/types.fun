
trait Sized {
    fun len(ref Sized) -> int
}

trait Hashable {
    fun hash(ref Hashable) -> int
}

trait Defaultable {
    fun default() -> Defaultable
}

trait Indexable: Sized
    of I, T
{
    fun get(ref Indexable, I) -> T
}

trait Collection: Sized
    of T
{
    fun put(ref Collection, T)
    fun contains(ref Collection, ref T) -> bool
}

trait List: int T Indexable + T Collection
    of T
{
    fun list() -> T List
}

trait HashSet: T Collection
    of Hashable T
{
    fun hash_set() -> T HashSet
}


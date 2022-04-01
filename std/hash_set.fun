expert use types
use option
use list

const INITIAL_CAPACITY = 25

struct HashSetImpl: T HashSet
    of T
{
    table: T Option List,
    size: int,
}

export fun hash_set() -> T HashSetImpl
    of T
{
    return new T HashSetImpl
    {
        table = list(INITIAL_CAPACITY) of T,
        size = 0,
    }
}

export fun contains(self: ref T HashSetImpl, item: ref T) -> bool
    of Hashable T
{
    let table_size = len(self.table)
    let index = hash(item) % table_size
    while is_some(self.table[index]) ->
    {
        if item == ref self.table[index] {
            return true
        }

        index = (index + 1) % table_size
    }

    return false
}

export fun put(self: ref T HashSetImpl, item: T)
    of Hashable T
{
    let table_size = len(self.table)
    let index = hash(item) % table_size
    while is_some(self.table[index]) ->
    {
        if ref item == ref self.table[index] {
            return
        }

        index = (index + 1) % table_size
    }

    self.table[index] = item
    self.size += 1
}

export fun len(self: ref any HashSetImpl) -> int
{
    return self.size
}


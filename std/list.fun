expert use types

extern printf
extern malloc
extern realloc
extern free

struct ListImpl: T List
    of T
{
    mem: ref T,
    size: int,
    capacity: int,
}

export fun list() -> T ListImpl
    of T
{
    return new T ListImpl
    {
        mem = 0,
        size = 0,
        capacity = 0,
    }
}

export fun list(size: int) -> T ListImpl
    of Defaultable T
{
    let mem = malloc(size * sizeof T) of ref T
    for i in 0..size {
        mem[i] = default() of T
    }

    return new T ListImpl
    {
        mem = mem,
        size = size,
        capacity = size,
    }
}

export fun put(self: ref T ListImpl, t: T)
    of T
{
    let index = self.size
    self.size = self.size + 1

    /*
    if self.size > self.capacity ->
    {
        self.capacity = self.capacity * 2
        self.mem = extern realloc(
            self.mem, self.capacity * sizeof t) of ref T
    }

    self.mem[index] = t
    */
}

export fun len(self: ref any ListImpl) -> int
{
    return self.size
}

export fun drop(self: any ListImpl)
{
    if self.mem > 0 -> {
        extern free(self.mem)
    }
}


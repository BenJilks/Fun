expert use types

export struct Option
    of T
{
    value: T,
    has_value: bool,
}

export some(value: T) -> T Option
    of T
{
    return new T Option
    {
        value = value,
        has_value = true,
    }
}

export none() -> T Option
    of T
{
    return new T Option
    {
        value = default() of T,
        has_value = true,
    }
}

export is_some(self: ref any Option) -> bool
{
    return self.has_value
}

export hash(self: ref Hashable Option) -> int
{
    if !is_some(self) ->
        return 0

    return hash(self.value)
}


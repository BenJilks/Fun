
extern printf
extern scanf

struct Vec of T
{
    x: T,
    y: T,
}

fun vec(x: T, y: T) -> T Vec
    of T
{
    return new T Vec
    {
        x = x,
        y = y,
    }
}

struct List of T
{
    items: ref T,
    capacity: int,
    size: int,
}

struct Optional of T
{
    value: T,
    has_value: bool,
}

fun some(value: T) -> T Optional
    of T
{
    return new T Optional
    {
        value = value,
        has_value = true,
    }
}

fun none(default: T) -> T Optional
    of T
{
    return new T Optional
    {
        value = default,
        has_value = false,
    }
}

fun has_value(optional: T Optional) -> bool
    of T
{
    return optional.has_value
}

fun list(first: T) -> T List
    of T
{
    return new T List
    {
        items = ref first,
        capacity = 1,
        size = 1,
    }
}

fun nth(self: T List, index: int) -> T Optional
    of T
{
    if index < 0 -> {
        return none(0)
    }
    if index > self.size -> {
        return none(0)
    }

    return some(deref self.items)
}

fun main()
{
    let test = list(2)
    let x = nth(test, 2)

    if has_value(x) -> printf("yes")
    else            -> printf("no")
}


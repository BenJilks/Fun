use std::list
use std::hash_set

fun print(i: int)
{
    extern printf("%d", i)
}

fun print(b: bool)
{
    if b -> extern printf("true")
    else -> extern printf("false")
}

fun is_nice(x: Sized) -> bool
{
    return len(x) == 69
}

fun main()
{
    let x = list() of int
    for i in 1..10 ->
        append(ref x, i)

    print(len(ref x))
    print(is_nice(ref x))
    drop(x)
}


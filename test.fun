
extern printf
extern scanf

struct Stuff
{
    foo: int,
    bar: int,
    baz: int,
}

struct Vec
{
    x: int,
    y: int,
    stuff: Stuff,
}

fun test() -> Vec
{
    return new Vec
    {
        x = 1,
        y = 2,
        stuff = new Stuff
        {
            foo = 3,
            bar = 4,
            baz = 5,
        },
    }
}

fun main()
{
}


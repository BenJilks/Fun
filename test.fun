
extern printf
extern scanf

struct Vec
{
    x: int,
    y: int,
}

fun vec(x: int, y: int) -> Vec
{
    return new Vec
    {
        x = x,
        y = y,
    }
}

fun add(lhs: Vec, rhs: Vec) -> Vec
{
    return new Vec
    {
        x = lhs.x + rhs.x,
        y = lhs.y + rhs.y,
    }
}

fun add(lhs: int, rhs: int) -> int
{
    return lhs + rhs
}

fun main()
{
    let a = vec(1, 2)
    let b = vec(3, 4)
    let c = add(a, b)
    printf("%d, %d%c", c.x, c.y, 10)
    printf("%d", add(2, 3))
}


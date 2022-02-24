
extern printf
extern scanf

struct Vec
{
    x: int,
    y: int,
}

fun main()
{
    let vec = new Vec { x = 1, y = 2 }
    printf("%d, %d", vec.x, vec.y)
}


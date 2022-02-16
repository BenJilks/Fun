
extern printf
extern scanf

func test(arr: int[3]) -> int[3]
{
    return [arr[2] + 1, arr[1] + 1, arr[0] + 1]
}

func main()
{
    let x = test([1, 2, 3])
    printf("%d, %d, %d", x[0], x[1], x[2])
}


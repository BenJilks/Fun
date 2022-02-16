global main
section .text

test:
push ebp
mov ebp, esp
; Return
mov dword [ebp+8], 1
mov dword [ebp+12], 2
mov dword [ebp+16], 3
mov esp, ebp
pop ebp
ret


main:
push ebp
mov ebp, esp
; Allocate return memory
sub esp, 12
; Call
call test
add esp, 0
; Copy 12 bytes ebp:-12 <- esp:0
mov eax, dword [esp]
mov dword [ebp-12], eax
mov eax, dword [esp+4]
mov dword [ebp-8], eax
mov eax, dword [esp+8]
mov dword [ebp-4], eax
; Free stack value
add esp, 12
; Return
xor eax, eax
mov esp, ebp
pop ebp
ret


section .data

extern printf
extern scanf


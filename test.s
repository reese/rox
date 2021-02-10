	.section	__TEXT,__text,regular,pure_instructions
	.macosx_version_min 10, 16
	.globl	_increment              ## -- Begin function increment
	.p2align	4, 0x90
_increment:                             ## @increment
	.cfi_startproc
## %bb.0:                               ## %entry
                                        ## kill: def $edi killed $edi def $rdi
	leal	1(%rdi), %eax
	retq
	.cfi_endproc
                                        ## -- End function
	.globl	_main                   ## -- Begin function main
	.p2align	4, 0x90
_main:                                  ## @main
	.cfi_startproc
## %bb.0:                               ## %entry
	pushq	%rax
	.cfi_def_cfa_offset 16
	movl	$5, %edi
	callq	_increment
	movl	%eax, %edi
	callq	_increment
	movl	$1, %eax
	popq	%rcx
	retq
	.cfi_endproc
                                        ## -- End function
.subsections_via_symbols

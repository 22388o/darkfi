" Syntax highlighting for zkas scripts.
" Symlink into ~/.vim/syntax/zk.vim

if exists("b:current_syntax")
    finish
endif

syn keyword zkasKeyword 
    \ constant
    \ contract
    \ circuit

syn keyword zkasType 
    \ EcPoint EcFixedPoint EcFixedPointBase EcFixedPointShort
    \ Base BaseArray Scalar ScalarArray
    \ MerklePath
    \ Uint32 Uint64

syn keyword zkasInstruction
    \ ec_add ec_mul ec_mul_base ec_mul_short
    \ ec_get_x ec_get_y
    \ base_add base_mul base_sub
    \ poseidon_hash calculate_merkle_root
    \ constrain_instance

syn region zkasString start='"' end='"' contained

syn keyword zkasTodo contained TODO FIXME XXX NOTE
syn match zkasComment "#.*$" contains=zkasTodo

hi def link zkasKeyword      Statement
hi def link zkasType         Type
hi def link zkasInstruction  Function
hi def link zkasString       Constant
hi def link zkasTodo         Todo
hi def link zkasComment      Comment

let b:current_syntax = "zk"

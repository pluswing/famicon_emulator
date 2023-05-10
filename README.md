# ファミコンエミュレータを作る!のリポジトリです。

## プレイリスト

https://www.youtube.com/playlist?list=PLp_EUEO9JJP1cMwbqzOHFOI9gPH_zoO0U

# document...

## OPERATIONS

```
- ADC SBC
- AND EOR ORA
- ASL LSR ROL ROR
- BCC BCS
- BEQ BNE
- BIT
- BMI BPL
- BRK ??
- BVC BVS
- CLC SEC
- CLD SED
- CLI SEI
- CLV
- CMP CPX CPY
- DEC DEX DEY
- INC INX INY
- JMP
- JSR RTS
- LDA LDX LDY
- NOP
- PHA PLA
- PHP PLP
- RTI ??
- STA STX STY
- TAX TXA
- TAY TYA
- TSX TXS
```

TODO
```
- cyclesの計算 => OK
- ppu write_to_dataをちゃんと実装する => OK
- brk命令の復活 => 0K
- CPU命令の未実装
- ppu 未実装
  - mask
  - oam
  - scroll (あとでOK)
- NMIまわり => OK
```

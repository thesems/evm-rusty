    /* "test/Counter.sol":58:463  contract Counter {... */
  mstore(0x40, 0x80)
  callvalue
  dup1
  iszero
  tag_1
  jumpi
  revert(0x00, 0x00)
tag_1:
  pop
  dataSize(sub_0)
  dup1
  dataOffset(sub_0)
  0x00
  codecopy
  0x00
  return
stop

sub_0: assembly {
        /* "test/Counter.sol":58:463  contract Counter {... */
      mstore(0x40, 0x80)
      callvalue
      dup1
      iszero
      tag_1
      jumpi
      revert(0x00, 0x00)
    tag_1:
      pop
      jumpi(tag_2, lt(calldatasize, 0x04))
      shr(0xe0, calldataload(0x00))
      dup1
      0x06661abd
      eq
      tag_3
      jumpi
      dup1
      0x371303c0
      eq
      tag_4
      jumpi
      dup1
      0x6d4ce63c
      eq
      tag_5
      jumpi
      dup1
      0xb3bcfa82
      eq
      tag_6
      jumpi
    tag_2:
      revert(0x00, 0x00)
        /* "test/Counter.sol":81:101  uint256 public count */
    tag_3:
      tag_7
      tag_8
      jump	// in
    tag_7:
      mload(0x40)
      tag_9
      swap2
      swap1
      tag_10
      jump	// in
    tag_9:
      mload(0x40)
      dup1
      swap2
      sub
      swap1
      return
        /* "test/Counter.sol":269:318  function inc() public {... */
    tag_4:
      tag_11
      tag_12
      jump	// in
    tag_11:
      stop
        /* "test/Counter.sol":149:223  function get() public view returns (uint256) {... */
    tag_5:
      tag_13
      tag_14
      jump	// in
    tag_13:
      mload(0x40)
      tag_15
      swap2
      swap1
      tag_10
      jump	// in
    tag_15:
      mload(0x40)
      dup1
      swap2
      sub
      swap1
      return
        /* "test/Counter.sol":364:461  function dec() public {... */
    tag_6:
      tag_16
      tag_17
      jump	// in
    tag_16:
      stop
        /* "test/Counter.sol":81:101  uint256 public count */
    tag_8:
      sload(0x00)
      dup2
      jump	// out
        /* "test/Counter.sol":269:318  function inc() public {... */
    tag_12:
        /* "test/Counter.sol":310:311  1 */
      0x01
        /* "test/Counter.sol":301:306  count */
      0x00
      0x00
        /* "test/Counter.sol":301:311  count += 1 */
      dup3
      dup3
      sload
      tag_19
      swap2
      swap1
      tag_20
      jump	// in
    tag_19:
      swap3
      pop
      pop
      dup2
      swap1
      sstore
      pop
        /* "test/Counter.sol":269:318  function inc() public {... */
      jump	// out
        /* "test/Counter.sol":149:223  function get() public view returns (uint256) {... */
    tag_14:
        /* "test/Counter.sol":185:192  uint256 */
      0x00
        /* "test/Counter.sol":211:216  count */
      sload(0x00)
        /* "test/Counter.sol":204:216  return count */
      swap1
      pop
        /* "test/Counter.sol":149:223  function get() public view returns (uint256) {... */
      swap1
      jump	// out
        /* "test/Counter.sol":364:461  function dec() public {... */
    tag_17:
        /* "test/Counter.sol":453:454  1 */
      0x01
        /* "test/Counter.sol":444:449  count */
      0x00
      0x00
        /* "test/Counter.sol":444:454  count -= 1 */
      dup3
      dup3
      sload
      tag_23
      swap2
      swap1
      tag_24
      jump	// in
    tag_23:
      swap3
      pop
      pop
      dup2
      swap1
      sstore
      pop
        /* "test/Counter.sol":364:461  function dec() public {... */
      jump	// out
        /* "#utility.yul":7:84   */
    tag_25:
        /* "#utility.yul":44:51   */
      0x00
        /* "#utility.yul":73:78   */
      dup2
        /* "#utility.yul":62:78   */
      swap1
      pop
        /* "#utility.yul":7:84   */
      swap2
      swap1
      pop
      jump	// out
        /* "#utility.yul":90:208   */
    tag_26:
        /* "#utility.yul":177:201   */
      tag_31
        /* "#utility.yul":195:200   */
      dup2
        /* "#utility.yul":177:201   */
      tag_25
      jump	// in
    tag_31:
        /* "#utility.yul":172:175   */
      dup3
        /* "#utility.yul":165:202   */
      mstore
        /* "#utility.yul":90:208   */
      pop
      pop
      jump	// out
        /* "#utility.yul":214:436   */
    tag_10:
        /* "#utility.yul":307:311   */
      0x00
        /* "#utility.yul":345:347   */
      0x20
        /* "#utility.yul":334:343   */
      dup3
        /* "#utility.yul":330:348   */
      add
        /* "#utility.yul":322:348   */
      swap1
      pop
        /* "#utility.yul":358:429   */
      tag_33
        /* "#utility.yul":426:427   */
      0x00
        /* "#utility.yul":415:424   */
      dup4
        /* "#utility.yul":411:428   */
      add
        /* "#utility.yul":402:408   */
      dup5
        /* "#utility.yul":358:429   */
      tag_26
      jump	// in
    tag_33:
        /* "#utility.yul":214:436   */
      swap3
      swap2
      pop
      pop
      jump	// out
        /* "#utility.yul":442:622   */
    tag_27:
        /* "#utility.yul":490:567   */
      0x4e487b7100000000000000000000000000000000000000000000000000000000
        /* "#utility.yul":487:488   */
      0x00
        /* "#utility.yul":480:568   */
      mstore
        /* "#utility.yul":587:591   */
      0x11
        /* "#utility.yul":584:585   */
      0x04
        /* "#utility.yul":577:592   */
      mstore
        /* "#utility.yul":611:615   */
      0x24
        /* "#utility.yul":608:609   */
      0x00
        /* "#utility.yul":601:616   */
      revert
        /* "#utility.yul":628:819   */
    tag_20:
        /* "#utility.yul":668:671   */
      0x00
        /* "#utility.yul":687:707   */
      tag_36
        /* "#utility.yul":705:706   */
      dup3
        /* "#utility.yul":687:707   */
      tag_25
      jump	// in
    tag_36:
        /* "#utility.yul":682:707   */
      swap2
      pop
        /* "#utility.yul":721:741   */
      tag_37
        /* "#utility.yul":739:740   */
      dup4
        /* "#utility.yul":721:741   */
      tag_25
      jump	// in
    tag_37:
        /* "#utility.yul":716:741   */
      swap3
      pop
        /* "#utility.yul":764:765   */
      dup3
        /* "#utility.yul":761:762   */
      dup3
        /* "#utility.yul":757:766   */
      add
        /* "#utility.yul":750:766   */
      swap1
      pop
        /* "#utility.yul":785:788   */
      dup1
        /* "#utility.yul":782:783   */
      dup3
        /* "#utility.yul":779:789   */
      gt
        /* "#utility.yul":776:812   */
      iszero
      tag_38
      jumpi
        /* "#utility.yul":792:810   */
      tag_39
      tag_27
      jump	// in
    tag_39:
        /* "#utility.yul":776:812   */
    tag_38:
        /* "#utility.yul":628:819   */
      swap3
      swap2
      pop
      pop
      jump	// out
        /* "#utility.yul":825:1019   */
    tag_24:
        /* "#utility.yul":865:869   */
      0x00
        /* "#utility.yul":885:905   */
      tag_41
        /* "#utility.yul":903:904   */
      dup3
        /* "#utility.yul":885:905   */
      tag_25
      jump	// in
    tag_41:
        /* "#utility.yul":880:905   */
      swap2
      pop
        /* "#utility.yul":919:939   */
      tag_42
        /* "#utility.yul":937:938   */
      dup4
        /* "#utility.yul":919:939   */
      tag_25
      jump	// in
    tag_42:
        /* "#utility.yul":914:939   */
      swap3
      pop
        /* "#utility.yul":963:964   */
      dup3
        /* "#utility.yul":960:961   */
      dup3
        /* "#utility.yul":956:965   */
      sub
        /* "#utility.yul":948:965   */
      swap1
      pop
        /* "#utility.yul":987:988   */
      dup2
        /* "#utility.yul":981:985   */
      dup2
        /* "#utility.yul":978:989   */
      gt
        /* "#utility.yul":975:1012   */
      iszero
      tag_43
      jumpi
        /* "#utility.yul":992:1010   */
      tag_44
      tag_27
      jump	// in
    tag_44:
        /* "#utility.yul":975:1012   */
    tag_43:
        /* "#utility.yul":825:1019   */
      swap3
      swap2
      pop
      pop
      jump	// out

    auxdata: 0xa26469706673582212207958b5856fde939fba18229736ca70305e2ddea2d5e60f2f4524713debe4f19f64736f6c634300081c0033
}

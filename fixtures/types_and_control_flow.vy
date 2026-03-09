# @version ^0.4.1

struct Position:
    x: uint256
    y: uint256

flag Access:
    READ
    WRITE

positions: HashMap[address, Position]

@internal
def _distance(a: uint256, b: uint256) -> uint256:
    if a >= b:
        return a - b
    return b - a

@view
@external
def manhattan(user: address) -> uint256:
    pos: Position = self.positions[user]
    return self._distance(pos.x, 0) + self._distance(pos.y, 0)

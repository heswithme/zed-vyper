"""Multiline call and expression coverage."""

event Ping: pass

@internal
def _scale(value: uint256) -> uint256:
    return value * 2

@view
@external
def sample(input: uint256) -> uint256:
    alpha: uint256 = staticcall self._scale(
        input +
        1
    )
    converted: int256 = -convert(
        alpha,
        int256,
    )
    log Ping()
    return alpha + convert(converted, uint256)

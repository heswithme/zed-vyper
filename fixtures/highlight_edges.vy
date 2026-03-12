"""Highlight edge coverage for query smoke tests."""

interface Decoder:
    def ping() -> uint256: view
    def send(target: address, payload: Bytes[2]) -> bool: nonpayable

event Seen:
    who: indexed(address)

@external
def sample(data: Bytes[32], target: address) -> decimal:
    raw: Bytes[2] = x"cafe"
    text: String[8] = r"vyper"
    blob: Bytes[2] = br"vy"
    note: String[8] = """hello"""
    values: DynArray[uint256, 4] = [1, 2, 3]
    pair: (address, uint256) = abi_decode(data, (address, uint256), unwrap_tuple=False)
    zero: address = empty(address)
    other: (address, uint256) = _abi_decode(data, (address, uint256))
    log Seen(target)
    return 0.5

@external
@view
def remote(target: address) -> uint256:
    return (staticcall
        Decoder(target).ping())

@external
def forward(target: address) -> bool:
    return (extcall
        Decoder(target).send(target, b"vy"))

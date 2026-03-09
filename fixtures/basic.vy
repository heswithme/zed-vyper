# @version ^0.4.1

owner: public(address)
total_supply: uint256

event Transfer:
    sender: indexed(address)
    receiver: indexed(address)
    value: uint256

@deploy
def __init__():
    self.owner = msg.sender

@external
def mint(receiver: address, value: uint256):
    assert msg.sender == self.owner, "not owner"
    self.total_supply += value
    log Transfer(msg.sender, receiver, value)

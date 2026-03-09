"""Module system coverage for Zed queries."""

import ownable
import access as access_module
from .interfaces import IERC20 as IERC20

implements: IERC20
uses: ownable
initializes: access_module
initializes: ownable[owner := access_module]

exports: (ownable.__interface__, access_module.__interface__)

event SetReceivers: pass

struct Receiver:
    account: address
    weight: uint256

receivers: DynArray[Receiver, 8]
admin: public(address)

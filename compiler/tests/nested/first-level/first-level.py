from typing import Literal

from ruu_runtime import Attribute, Rule


class CompanyMemberRule(Rule):
    relationship: Literal["member"]
    grants: tuple[Literal["write.basic", "read"]]
    attributes: tuple[()]
    rules: tuple["UserCompanyRule | UserSiblingRule", ...]

class UserCompanyAuthorizationAttr(Attribute):
    name: Literal["authorization"]

class UserCompanyUserTypeAttr(Attribute):
    name: Literal["user-type"]

class UserCompanyRule(Rule):
    relationship: Literal["company"]
    grants: tuple[Literal["read"]]
    attributes: tuple["UserCompanyAuthorizationAttr | UserCompanyUserTypeAttr", ...]
    rules: tuple["CompanyMemberRule", ...]

class UserSiblingRule(Rule):
    relationship: Literal["sibling"]
    grants: tuple[Literal["write.basic", "read"]]
    attributes: tuple[()]
    rules: tuple["UserCompanyRule | UserSiblingRule", ...]

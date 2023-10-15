# pyright: reportPrivateUsage=false, reportIncompatibleVariableOverride=false

from dataclasses import dataclass
from dataclasses import field as dc_field
from typing import Any, Callable, Generator, Literal, TypeVar, cast

from pydantic import BaseModel
from pydantic import Field as pyd_field

Permission = tuple[str, ...]

_OnRegister = cast(Any, ...)


class RegistryModel(BaseModel):
    # This is a class parameter, not an instance parameter.
    # It's used to store the registry instance for lookup
    # and resolution.
    _registry: "Registry | None" = None

    @classmethod
    def set_registry(cls, registry: "Registry") -> None:
        cls._registry = registry

    def __hash__(self) -> int:
        sorted_values = sorted(
            ((key, value) for key, value in self.__dict__.items() if key[0] != "_"),
            key=lambda x: x[0],
        )
        hash_elements = (self.__class__,) + tuple(sorted_values)

        return hash(hash_elements)


class Attribute(RegistryModel):
    name: str
    arguments: tuple[str, ...] = pyd_field(default_factory=tuple)

    _entity: str = _OnRegister
    _relationship: str = _OnRegister
    _attribute: str = _OnRegister


class Rule(RegistryModel):
    relationship: str
    grants: tuple[Permission, ...]
    attributes: tuple[Attribute, ...]
    rules: "tuple[Rule, ...]"

    include_fragments: tuple[str, ...] = pyd_field(default_factory=tuple)

    _src_entity: str = _OnRegister
    _relationship: str = _OnRegister
    _dst_entity: str = _OnRegister

    @property
    def resolved_fragments(self) -> "Generator[Fragment, None, None]":
        if not self._registry:
            return None

        for fragment in self.include_fragments:
            resolved_fragment = self._registry.resolve_fragment(
                self._dst_entity, fragment
            )

            if resolved_fragment:
                yield resolved_fragment

    @property
    def resolved_grants(self) -> set[Permission]:
        grants = set(self.grants)

        for fragment in self.resolved_fragments:
            for grant in fragment.grants:
                grants.add(grant)

        return grants

    @property
    def resolved_rules(self) -> "Generator[Rule, None, None]":
        yield from self.rules

        for fragment in self.resolved_fragments:
            yield from fragment.rules


class Universal(Rule):
    relationship: Literal["*"]


class Fragment(RegistryModel):
    grants: tuple[Permission, ...]
    rules: tuple[Rule, ...]

    _entity: str = _OnRegister
    _fragment: str = _OnRegister


class Entrypoint(RegistryModel):
    entrypoint: str
    rules: tuple[Rule, ...]


class Schema(RegistryModel):
    entrypoints: tuple[Entrypoint, ...] = pyd_field(default_factory=tuple)
    fragments: tuple[Fragment, ...] = pyd_field(default_factory=tuple)

    def register_globals(self) -> None:
        assert self._registry

        for fragment in self.fragments:
            self._registry.register_fragment_singleton(
                fragment._entity, fragment._fragment, fragment
            )


@dataclass
class RegistryRule:
    rule: type[Rule] | None = None
    attributes: dict[str, type[Attribute]] = dc_field(default_factory=dict)


@dataclass
class RegistryFragment:
    fragment: type[Fragment] | None = None
    fragment_singleton: Fragment | None = None


@dataclass
class RegistryEntity:
    fragments: dict[str, RegistryFragment] = dc_field(default_factory=dict)
    rules: dict[str, RegistryRule] = dc_field(default_factory=dict)


_T = TypeVar("_T", bound=type[RegistryModel])
_F = TypeVar("_F", bound=type[Fragment])
_R = TypeVar("_R", bound=type[Rule])
_A = TypeVar("_A", bound=type[Attribute])


@dataclass
class Registry:
    entities: dict[str, RegistryEntity] = dc_field(default_factory=dict)
    elements: set[type[RegistryModel]] = dc_field(default_factory=set)

    def register_fragment(self, entity: str, fragment: str) -> Callable[[_F], _F]:
        def wrapper(cls: _F) -> _F:
            reg_entity = self.entities.get(entity, RegistryEntity())
            reg_fragment = reg_entity.fragments.get(fragment, RegistryFragment())

            self.entities[entity] = reg_entity
            reg_entity.fragments[fragment] = reg_fragment

            reg_fragment.fragment = cls

            cls._entity = entity
            cls._fragment = fragment

            return self.bind(cls)

        return wrapper

    def register_fragment_singleton(self, entity: str, fragment: str, inst: Fragment) -> None:
        reg_entity = self.entities.get(entity, RegistryEntity())
        reg_fragment = reg_entity.fragments.get(fragment, RegistryFragment())

        self.entities[entity] = reg_entity
        reg_entity.fragments[fragment] = reg_fragment

        reg_fragment.fragment_singleton = inst

    def resolve_fragment(self, entity: str, fragment: str) -> Fragment | None:
        if entity not in self.entities:
            return None

        reg_entity = self.entities[entity]

        if fragment not in reg_entity.fragments:
            return None

        reg_fragment = reg_entity.fragments[fragment]
        return reg_fragment.fragment_singleton

    def register_relationship(
        self, src_entity: str, relationship: str, dst_entity: str
    ) -> Callable[[_R], _R]:
        def wrapper(cls: _R) -> _R:
            reg_entity = self.entities.get(src_entity, RegistryEntity())
            reg_rule = reg_entity.rules.get(relationship, RegistryRule())

            self.entities[src_entity] = reg_entity
            reg_entity.rules[relationship] = reg_rule

            reg_rule.rule = cls

            cls._src_entity = src_entity
            cls._relationship = relationship
            cls._dst_entity = dst_entity

            return self.bind(cls)

        return wrapper

    def register_attribute(self, entity: str, relationship: str, attribute: str) -> Callable[[_A], _A]:
        def wrapper(cls: _A) -> _A:
            reg_entity = self.entities.get(entity, RegistryEntity())
            reg_rule = reg_entity.rules.get(relationship, RegistryRule())

            self.entities[entity] = reg_entity
            reg_entity.rules[relationship] = reg_rule

            reg_rule.attributes[attribute] = cls

            cls._entity = entity
            cls._relationship = relationship

            return self.bind(cls)

        return wrapper

    def bind(self, cls: _T) -> _T:
        cls.set_registry(self)
        self.elements.add(cls)

        return cls

    def update_annotations(self) -> None:
        for element in self.elements:
            element.model_rebuild()


registry: Registry = Registry()

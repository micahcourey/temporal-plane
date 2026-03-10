from mnemix import (
    Mnemix,
    MnemixBinaryNotFoundError,
    MnemixCommandError,
    MnemixDecodeError,
    MnemixError,
    RememberRequest,
)
from mnemix import Mnemix


def test_mnemix_alias_points_at_mnemix_client() -> None:
    assert Mnemix is Mnemix


def test_mnemix_request_models_are_exported() -> None:
    request = RememberRequest(
        id="mem-001",
        scope="scope-1",
        kind="observation",
        title="title",
        summary="summary",
        detail="detail",
    )
    assert request.id == "mem-001"


def test_mnemix_error_aliases_exist() -> None:
    assert issubclass(MnemixError, Exception)
    assert issubclass(MnemixCommandError, MnemixError)
    assert issubclass(MnemixBinaryNotFoundError, MnemixError)
    assert issubclass(MnemixDecodeError, MnemixError)
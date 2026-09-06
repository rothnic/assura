from agent_init_python import stable_value


def test_stable_value_is_preserved() -> None:
    assert stable_value() == 7

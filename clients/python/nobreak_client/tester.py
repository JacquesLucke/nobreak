from __future__ import annotations
from .client import NobreakClient, NobreakOperationMode
from .value_handler import ValueHandler, SuccessCompareResult, FailCompareResult
from .default_value_handlers import get_default_value_handler


class NobreakTester:
    def __init__(self, client: NobreakClient, parent_key: list[str] | None = None):
        self.client = client
        self.parent_key = [] if parent_key is None else parent_key

    def test(
        self, sub_key: str, value: object, value_handler: ValueHandler | None = None
    ):
        if value_handler is None:
            value_handler = get_default_value_handler(type(value))
            if value_handler is None:
                raise RuntimeError(
                    f"There is no default value handler for {type(value)}"
                )

        key = self.parent_key + [sub_key]
        if self.client.operation_mode == NobreakOperationMode.UPDATE:
            value_bytes = value_handler.serialize(value)
            self.client.log(key, value_bytes)
        elif self.client.operation_mode == NobreakOperationMode.CHECK:
            stored_value = self.client.get(key)
            if stored_value is None:
                print("Value was not stored")
            else:
                compare_result = value_handler.compare(value, stored_value)
                if isinstance(compare_result, SuccessCompareResult):
                    print("Equal:", sub_key, value)
                elif isinstance(compare_result, FailCompareResult):
                    print(compare_result.message)
                    self.client.fail(key, compare_result.message)
                else:
                    raise RuntimeError(
                        "Value handler did not return a valid `CompareResult`."
                    )
        else:
            raise RuntimeError("Unknown nobreak operation mode.")

    def sub(self, sub_key: str) -> NobreakTester | None:
        return NobreakTester(self.client, self.parent_key + [sub_key])

// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { JsonValue } from "./serde_json/JsonValue";

export type AnimationConfig<Initial = JsonValue, Animate = JsonValue, Exit = JsonValue, Variant = JsonValue, Transition = JsonValue> = { initial: Initial | null, animate: Animate | null, exit: Exit | null, variants: Variant | null, transition: Transition | null, inherit_children: boolean, };

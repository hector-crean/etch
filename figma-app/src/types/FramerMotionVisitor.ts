// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { AnimationConfig } from "./AnimationConfig";
import type { JsonValue } from "./serde_json/JsonValue";

export type FramerMotionVisitor<Initial = JsonValue, Animate = JsonValue, Exit = JsonValue, Variant = JsonValue, Transition = JsonValue> = { animations: { [key in string]?: AnimationConfig<Initial, Animate, Exit, Variant, Transition> }, current_parent_id: string | null, current_parent_config: AnimationConfig<Initial, Animate, Exit, Variant, Transition> | null, inside_svg: boolean, };

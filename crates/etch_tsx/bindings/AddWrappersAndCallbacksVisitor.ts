// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { Callback } from "./Callback";
import type { ComponentWrapper } from "./ComponentWrapper";

/**
 * A visitor that adds event handlers to JSX elements and transforms JSX structure
 */
export type AddWrappersAndCallbacksVisitor = { callbacks: { [key in string]?: Array<Callback> }, component_wrappers: { [key in string]?: ComponentWrapper }, action_imports: { [key in string]?: Array<string> }, };

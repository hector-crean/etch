// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { Directory } from "./Directory";
import type { File } from "./File";

export type AppRouterEntry<T> = { "type": "Directory" } & Directory<T> | { "type": "File" } & File<T>;

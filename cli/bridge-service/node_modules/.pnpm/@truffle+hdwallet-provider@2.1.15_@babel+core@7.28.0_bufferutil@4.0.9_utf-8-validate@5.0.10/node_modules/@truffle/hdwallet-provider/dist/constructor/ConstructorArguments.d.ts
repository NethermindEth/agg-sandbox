import type * as LegacyConstructor from "./LegacyConstructor";
import type * as Constructor from "./Constructor";
export type ConstructorArguments = LegacyConstructor.Arguments | [Constructor.InputOptions];

export enum ItemType {
  Struct = "struct",
  Enum = "enum",
  Route = "route",
}

export interface FieldInfo {
  name: string;
  doc: string | null;
  field_type: string;
  flattened: boolean;
  nullable: boolean;
  ommitable: boolean;
}

export interface StructInfo {
  type: ItemType.Struct;
  name: string;
  doc: string | null;
  fields: FieldInfo[];
}

export enum EnumType {
  Unit = "unit",
  Tuple = "tuple",
  Struct = "struct",
}

export interface UnitEnumVariant {
  type: EnumType.Unit
  name: string;
  doc: string | null;
}

export interface TupleEnumVariant {
  type: EnumType.Tuple
  name: string;
  doc: string | null;
  field_type: string;
}

export interface StructEnumVariant extends FieldInfo {
  type: EnumType.Struct;
}

export type EnumVariant = UnitEnumVariant | TupleEnumVariant | StructEnumVariant;

export interface EnumInfo {
  name: string;
  doc: string | null;
  tag: string | null;
  untagged: boolean;
  content: string | null;
  rename_all: string | null;
  variants: EnumVariant[];
}

export interface PathParamInfo {
  name: string;
  param_type: string;
}

export interface QueryParamInfo {
  name: string;
  param_type: string;
}

export interface RouteInfo {
  type: ItemType.Route;
  name: string;
  method: string;
  route: string;
  doc: string | null;
  path_params: PathParamInfo[];
  query_params: QueryParamInfo[];
  body_type: string | null;
  return_type: string | null;
  guards: string[];
}


export type ItemInfo = StructInfo | EnumInfo | RouteInfo;

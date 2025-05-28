export enum ItemType {
  Object = 'object',
  Enum = 'enum',
  Route = 'route'
}

export interface FieldInfo {
  name: string;
  doc: string | null;
  type: string;
  flattened: boolean;
  nullable: boolean;
  omittable: boolean;
}

export interface ObjectInfo {
  type: ItemType.Object;
  fields: FieldInfo[];
}

export enum VariantType {
  Unit = 'unit',
  Tuple = 'tuple',
  Object = 'object'
}

export interface UnitEnumVariant {
  type: VariantType.Unit;
  name: string;
  doc: string | null;
}

export interface TupleEnumVariant {
  type: VariantType.Tuple;
  name: string;
  doc: string | null;
  field_type: string;
}

export interface StructEnumVariant {
  type: VariantType.Object;
  name: string;
  doc: string | null;
  fields: FieldInfo[];
}

export type EnumVariant = UnitEnumVariant | TupleEnumVariant | StructEnumVariant;

export interface EnumInfo {
  type: ItemType.Enum;
  tag: string | null;
  untagged: boolean;
  content: string | null;
  variants: EnumVariant[];
}

export interface PathParamInfo {
  name: string;
  type: string;
}

export interface QueryParamInfo {
  name: string;
  type: string;
  nullable: boolean;
}

export interface Body {
  type: string;
  format: string;
}

export interface Response {
  type: string;
  format: string;
  status_code: number;
  rate_limits: boolean;
}

export interface RouteInfo {
  type: ItemType.Route;
  method: string;
  route: string;
  path_params: PathParamInfo[];
  query_params: QueryParamInfo[];
  body: Body | null;
  response: Response | null;
  requires_auth?: boolean;
}

export type Item = ObjectInfo | EnumInfo | RouteInfo;

export interface ItemInfo {
  name: string;
  doc: string;
  category: string;
  hidden: boolean;
  package: string;
  item: Item;
}

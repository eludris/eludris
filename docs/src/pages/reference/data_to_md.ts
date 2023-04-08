import { readFileSync } from 'fs';
import {
  EnumInfo,
  EnumVariant,
  FieldInfo,
  ItemInfo,
  ItemType,
  StructInfo,
  VariantType,
  RouteInfo
} from '../../lib/types';
import AUTODOC_ENTRIES from '../../../public/autodoc/index.json';

export default (info: ItemInfo): string => {
  let content = `# ${uncodeName(info.name)}`;
  let example = '';
  if (info.item.type == ItemType.Route) {
    content += `\n\n<span class="method">${info.item.method
      }</span><span class="route">${info.item.route.replace(
        /<.+?>/gm,
        '<span class="special-segment">$&</span>'
      )}</span>`;
  }
  if (info.doc) {
    const parts = info.doc.split('-----');
    let doc = parts.shift();
    example = parts.join('-----');
    content += `\n\n${displayDoc(doc)}`;
  }
  if (info.item.type == ItemType.Struct) {
    content += `\n\n${displayFields(info.item.fields)}`;
  } else if (info.item.type == ItemType.Enum) {
    info.item.variants.forEach((variant) => {
      content += `\n## ${uncodeName(variant.name)}`;
      let variant_example = '';
      if (variant.doc) {
        const parts = variant.doc.split('-----');
        let doc = parts.shift();
        variant_example = parts.join('-----');
        content += `\n\n${displayDoc(doc)}`;
      }
      content += `\n${displayVariant(variant, <EnumInfo>info.item)}`;
      if (variant_example) {
        content += `\n${example}`;
      }
    });
  } else {
    content += `\n\n${displayRoute(info.item)}`;
  }
  if (example) {
    content += `\n${example}`;
  }
  return content;
};

const briefItem = (item: StructInfo | EnumInfo): string => {
  if (item.type == ItemType.Struct) {
    if (!item.fields.length) {
      return '';
    }
    return displayFields(item.fields);
  } else {
    console.log(item);
    let content = '';
    item.variants.forEach((variant) => {
      content += `\n- ${uncodeName(variant.name)}\n\n${variant.doc ?? ''}`;
      content += `\n${displayVariant(variant, item)}`;
    });
    return content;
  }
};

const displayFields = (fields: FieldInfo[]): string => {
  if (!fields.length) {
    return '';
  }
  let content = '|Field|Type|Description|\n|---|---|---|';
  fields.forEach((field) => {
    content += `\n${displayField(field)}`;
  });
  return content;
};

const displayField = (field: FieldInfo): string => {
  const innerType =
    field.flattened && AUTODOC_ENTRIES.find((entry) => entry.endsWith(`/${field.name}.json`));
  if (innerType) {
    let innerData: StructInfo = JSON.parse(readFileSync(`public/autodoc/${innerType}`).toString());
    let fields = '';
    innerData.fields.forEach((field) => {
      fields += `\n\n${displayField(field)}`;
    });
    return fields;
  }
  return `|${field.name}${field.ommitable ? '?' : ''}|${displayType(field.field_type)}${field.nullable ? '?' : ''
    }|${displayDoc(field.doc).replace(/(\S)\n(\S)/gm, '$1 $2')}|`;
};

const displayVariant = (variant: EnumVariant, item: EnumInfo): string => {
  let content = '';
  if (variant.type == VariantType.Unit) {
    if (item.tag) {
      content += `\n\n|Field|Type|Description|\n|---|---|---|\n|${item.tag}|"${switchCase(
        variant.name,
        item.rename_all
      )}"|The tag of this variant`;
    }
  } else if (variant.type == VariantType.Tuple) {
    if (item.tag) {
      content += `\n\n|Field|Type|Description|\n|---|---|---|\n|${item.tag}|"${switchCase(
        variant.name,
        item.rename_all
      )}"|The tag of this variant`;
      if (item.content) {
        content += `\n|${item.content}|${displayType(variant.field_type)}|The data of this variant`;
      }
    } else {
      content += `This variant contains a ${displayType(variant.field_type)}`;
      const innerType = AUTODOC_ENTRIES.find((entry) =>
        entry.endsWith(`/${variant.field_type}.json`)
      );
      if (innerType) {
        let innerData: StructInfo | EnumInfo = JSON.parse(
          readFileSync(`public/autodoc/${innerType}`).toString()
        ).item;
        content += `\n\n${briefItem(innerData)}`;
      }
    }
  } else if (variant.type == VariantType.Struct) {
    content += '\n\n|Field|Type|Description|\n|---|---|---|';
    if (item.tag) {
      content += `\n|${item.tag}|"${switchCase(
        variant.name,
        item.rename_all
      )}"|The tag of this variant|`;
      if (item.content) {
        content += `\n|${item.content}|${uncodeName(variant.name)} Data|The data of this variant`;
        content += '\n\nWith the data of this variant being:';
        content += `\n\n${displayFields(variant.fields)}`;
      } else {
        variant.fields.forEach((field) => {
          content += `\n${displayField(field)}`;
        });
      }
    }
  }
  return content;
};

const displayRoute = (item: RouteInfo): string => {
  let content = '';
  if (item.path_params.length) {
    content += '\n\n## Path Params\n\n|Name|Type|\n|---|---|';
    item.path_params.forEach((param) => {
      content += `\n|${param.name}|${displayType(param.param_type)}|`;
    });
  }
  if (item.query_params.length) {
    content += '\n\n## Query Params\n\n|Name|Type|\n|---|---|';
    item.query_params.forEach((param) => {
      content += `\n|${param.name}|${displayType(param.param_type)}|`;
    });
  }
  if (item.body_type) {
    content += '\n\n## Request Body';
    let body_type = item.body_type.replace(/Json<(.+)>/gm, '$1');
    content += `\n\n${displayType(body_type)}`;
    const innerType = AUTODOC_ENTRIES.find((entry) => entry.endsWith(`/${body_type}.json`));
    if (innerType) {
      let innerData: StructInfo | EnumInfo = JSON.parse(
        readFileSync(`public/autodoc/${innerType}`).toString()
      ).item;
      content += `\n\n${briefItem(innerData)}`;
    }
  }
  if (item.return_type) {
    content += '\n\n## Response';
    let return_type = item.return_type
      .replace(/Result<(.+?), .+?>/gm, '$1')
      .replace(/RateLimitedRouteResponse<(.+?)>/gm, '$1')
      .replace(/Json<(.+?)>/gm, '$1');
    content += `\n\n${displayType(return_type)}`;
    const innerType = AUTODOC_ENTRIES.find((entry) => entry.endsWith(`/${return_type}.json`));
    if (innerType) {
      let innerData: StructInfo | EnumInfo = JSON.parse(
        readFileSync(`public/autodoc/${innerType}`).toString()
      ).item;
      content += `\n\n${briefItem(innerData)}`;
    }
  }
  return content.substring(2); // to remove the first double newline
};

const displayDoc = (doc: string | null | undefined): string => {
  return doc
    ? doc.replace(/\[`(.+)`\]/gm, (_, p1) => {
      return `[${p1}](/reference/${AUTODOC_ENTRIES.find((entry) => entry.endsWith(`/${p1}.json`))?.split('.')[0]
        })`;
    })
    : '';
};

const switchCase = (content: string, new_case: string | null): string => {
  if (new_case == 'SCREAMING_SNAKE_CASE') {
    return content.replace(/(\S)([A-Z])/gm, '$1_$2').toUpperCase();
  }
  return content;
};

const displayType = (type: string): string => {
  if (type == 'u32' || type == 'u64' || type == 'usize') {
    return 'Number';
  }
  type = type
    .replace(/Option<(.+)>/gm, '$1')
    .replace(/Json<(.+)>/gm, '$1')
    .replace(/</gm, '\\<');
  let entry = AUTODOC_ENTRIES.find((entry) => entry.endsWith(`/${type}.json`))?.split('.')[0];
  return entry ? `[${type}](/reference/${entry})` : type;
};

const uncodeName = (name: string): string => {
  return name
    .replace(/(?:^|_)([a-z0-9])/gm, (_, p1: string) => p1.toUpperCase())
    .replace(/[A-Z]/gm, ' $&');
};

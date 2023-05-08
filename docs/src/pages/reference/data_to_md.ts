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

const TYPE_OVERRIDES: Record<string, string> = {
  FetchResponse: 'Raw file content.'
};

export default (info: ItemInfo): string => {
  let content = `# ${uncodeName(info.name)}`;
  let example = '';
  if (info.item.type == ItemType.Route) {
    const route = info.item.route.replace(/<.+?>/gm, (match) => {
      // Replace angle brackets with HTML character entities
      return match.replace('<', '&lt;').replace('>', '&gt;');
    });
    content += `\n\n<span class="method">${
      info.item.method
    }</span><span class="route">${route.replace(
      /&lt;*.+?&gt;/gm,
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
      content += `\n${displayVariant(variant, <EnumInfo>info.item, info.name)}`;
      if (variant_example) {
        content += `\n${variant_example}`;
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

const briefItem = (item: StructInfo | EnumInfo, model: string): string => {
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
      content += `\n${displayVariant(variant, item, model)}`;
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
    field.flattened && AUTODOC_ENTRIES.find((entry) => entry.endsWith(`/${field.field_type}.json`));
  if (innerType) {
    let innerData: StructInfo = JSON.parse(
      readFileSync(`public/autodoc/${innerType}`).toString()
    ).item;
    let fields = '';
    innerData.fields.forEach((field) => {
      fields += `${displayField(field)}\n`;
    });
    return fields.trim();
  }
  return `|${field.name}${field.ommitable ? '?' : ''}|${displayType(field.field_type)}${
    field.nullable ? '?' : ''
  }|${displayInlineDoc(field.doc)}|`;
};

const getTagDescription = (tag: string, model: string): string => {
  return `The ${tag} of this ${model} variant.`;
};

const displayVariant = (variant: EnumVariant, item: EnumInfo, model: string): string => {
  let content = '';
  if (variant.type == VariantType.Unit) {
    if (item.tag) {
      content += `\n\n|Field|Type|Description|\n|---|---|---|\n|${item.tag}|"${switchCase(
        variant.name,
        item.rename_all
      )}"|${getTagDescription(item.tag, model)}`;
    }
  } else if (variant.type == VariantType.Tuple) {
    if (item.tag) {
      content += `\n\n|Field|Type|Description|\n|---|---|---|\n|${item.tag}|"${switchCase(
        variant.name,
        item.rename_all
      )}"|${getTagDescription(item.tag, model)}`;
      if (item.content) {
        content += `\n|${item.content}|${displayType(variant.field_type)}|The data of this variant`;
      }
    } else {
      content += `This variant contains a ${displayType(variant.field_type)}`;
      const innerType = AUTODOC_ENTRIES.find((entry) =>
        entry.endsWith(`/${variant.field_type}.json`)
      );
      if (innerType) {
        let data = JSON.parse(readFileSync(`public/autodoc/${innerType}`).toString());
        content += `\n\n${briefItem(data.item, data.name)}`;
      }
    }
  } else if (variant.type == VariantType.Struct) {
    content += '\n\n|Field|Type|Description|\n|---|---|---|';
    if (item.tag) {
      let desc = getTagDescription(item.tag, model);
      content += `\n|${item.tag}|"${switchCase(variant.name, item.rename_all)}|${desc}`;
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
    let body_type = item.body_type;
    if (body_type.startsWith('Json<')) {
      content += `\n\nA JSON ${displayType(body_type)}`;
    } else if (body_type.startsWith('Form<')) {
      content += `\n\nA \`multipart/form-data\` ${displayType(body_type)}`;
    } else {
      content += `\n\n${displayType(body_type)}`;
    }

    const innerType = AUTODOC_ENTRIES.find((entry) => entry.endsWith(`/${body_type}.json`));
    if (innerType) {
      let data: any = JSON.parse(readFileSync(`public/autodoc/${innerType}`).toString());
      content += `\n\n${briefItem(data.item, data.name)}`;
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
      let data: any = JSON.parse(readFileSync(`public/autodoc/${innerType}`).toString());
      content += `\n\n${briefItem(data.item, data.name)}`;
    }
  }
  return content.substring(2); // to remove the first double newline
};

const displayDoc = (doc: string | null | undefined): string => {
  return doc ?? '';
};

const displayInlineDoc = (doc: string | null | undefined): string => {
  return displayDoc(doc)
    .replace(/\n{2,}/gm, '<br><br>')
    .replace(/(\S)\n(\S)/gm, '$1 $2');
};

const switchCase = (content: string, new_case: string | null): string => {
  if (new_case == 'SCREAMING_SNAKE_CASE') {
    return content.replace(/(\S)([A-Z])/gm, '$1_$2').toUpperCase();
  }
  return content;
};

const displayType = (type: string): string => {
  type = type
    .replace(/Option<(.+)>/gm, '$1')
    .replace(/Json<(.+)>/gm, '$1')
    .replace(/Form<(.+)>/gm, '$1')
    .replace(/</gm, '\\<');

  if (type == 'u32' || type == 'u64' || type == 'usize') {
    return 'Number';
  } else if (type == 'bool') {
    return 'Boolean';
  } else if (type == 'str') {
    return 'String';
  } else if (type == 'TempFile') {
    return 'File';
  } else if (type in TYPE_OVERRIDES) {
    return TYPE_OVERRIDES[type];
  }

  let entry = AUTODOC_ENTRIES.find((entry) => entry.endsWith(`/${type}.json`))?.split('.')[0];
  return entry ? `[${type}](/reference/${entry})` : type;
};

const uncodeName = (name: string): string => {
  return name
    .replace(/(?:^|_)([a-z0-9])/gm, (_, p1: string) => p1.toUpperCase())
    .replace(/[A-Z]/gm, ' $&')
    .trim();
};

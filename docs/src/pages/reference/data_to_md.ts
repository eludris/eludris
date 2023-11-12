import { readFileSync } from 'fs';
import {
  EnumInfo,
  EnumVariant,
  FieldInfo,
  ItemInfo,
  ItemType,
  ObjectInfo,
  VariantType,
  RouteInfo,
  Item
} from '../../lib/types';
import AUTODOC_ENTRIES from '../../../public/autodoc/index.json';

export default (info: ItemInfo): string => {
  let content = `# ${uncodeName(info.name)}`;
  let example = '';
  if (info.item.type == ItemType.Route) {
    // Replace angle brackets with HTML character entities
    const route = info.item.route.replace('<', '&lt;').replace('>', '&gt;');
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
  if (info.item.type == ItemType.Object) {
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

const briefItem = (item: Item, model: string): string => {
  if (item.type == ItemType.Object) {
    if (!item.fields.length) {
      return '';
    }
    return displayFields(item.fields);
  } else if (item.type == ItemType.Enum) {
    let content = '';
    item.variants.forEach((variant) => {
      content += `\n- ${uncodeName(variant.name)}\n\n${variant.doc ?? ''}`;
      content += `\n${displayVariant(variant, item, model)}`;
    });
    return content;
  } else {
    throw new Error(`Unexpected item type: ${item.type}`);
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
    field.flattened && AUTODOC_ENTRIES.items.find((entry) => entry.endsWith(`/${field.type}.json`));
  if (innerType) {
    let innerData: ObjectInfo = JSON.parse(
      readFileSync(`public/autodoc/${innerType}`).toString()
    ).item;
    let fields = '';
    innerData.fields.forEach((field) => {
      fields += `${displayField(field)}\n`;
    });
    return fields.trim();
  }
  return `|${field.name}${field.omittable ? '?' : ''}|${displayType(field.type)}${
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
      let desc = getTagDescription(item.tag, model);
      content += `\n\n|Field|Type|Description|\n|---|---|---|\n|${item.tag}|"${variant.name}"|${desc}`;
    }
  } else if (variant.type == VariantType.Tuple) {
    if (item.tag) {
      let desc = getTagDescription(item.tag, model);
      content += `\n\n|Field|Type|Description|\n|---|---|---|\n|${item.tag}|"${variant.name}"|${desc}`;
      if (item.content) {
        content += `\n|${item.content}|${displayType(variant.field_type)}|The data of this variant`;
      }
    } else {
      content += `This variant contains a ${displayType(variant.field_type)}`;
      const innerType = AUTODOC_ENTRIES.items.find((entry) =>
        entry.endsWith(`/${variant.field_type}.json`)
      );
      if (innerType) {
        let data: ItemInfo = JSON.parse(readFileSync(`public/autodoc/${innerType}`).toString());
        content += `\n\n${briefItem(data.item, data.name)}`;
      }
    }
  } else if (variant.type == VariantType.Object) {
    content += '\n\n|Field|Type|Description|\n|---|---|---|';
    if (item.tag) {
      let desc = getTagDescription(item.tag, model);
      content += `\n|${item.tag}|"${variant.name}"|${desc}`;
      if (item.content) {
        content += `\n|${item.content}|${uncodeName(variant.name)} Data|The data of this variant.`;
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
      content += `\n|${param.name}|${displayType(param.type)}|`;
    });
  }
  if (item.query_params.length) {
    content += '\n\n## Query Params\n\n|Name|Type|\n|---|---|';
    item.query_params.forEach((param) => {
      content += `\n|${param.name}|${displayType(param.type)}|`;
    });
  }
  if (item.body) {
    content += '\n\n## Request Body\n\n';
    let body_type = item.body.type;
    let format: string | null = null;
    if (item.body.format == 'application/json') {
      format = 'JSON';
    } else if (item.body.format == 'multipart/form-data') {
      format = 'Multi-part form data';
    }

    body_type = body_type.replace(/Json<(.+?)>/gm, '$1').replace(/Form<(.+?)>/gm, '$1');
    const innerType = AUTODOC_ENTRIES.items.find((entry) => entry.endsWith(`/${body_type}.json`));
    if (innerType) {
      let data: ItemInfo = JSON.parse(readFileSync(`public/autodoc/${innerType}`).toString());
      if (!data.hidden) {
        if (format) {
          content += `A ${format} ${displayType(body_type)}`;
        } else {
          content += displayType(body_type);
        }
      }

      content += `\n\n${briefItem(data.item, data.name)}`;
    } else {
      content += displayType(body_type);
    }
  }
  if (item.response) {
    content += `\n\n## Response\n\n<span class="status">${item.response.status_code}</span>`;
    let response_type = item.response.type;
    if (item.response.format == 'raw') {
      content += 'Raw file content.';
    } else {
      content += `${displayType(response_type)}`;
      const innerType = AUTODOC_ENTRIES.items.find((entry) =>
        entry.endsWith(`/${response_type}.json`)
      );
      if (innerType) {
        let data: ItemInfo = JSON.parse(readFileSync(`public/autodoc/${innerType}`).toString());
        content += `\n\n${briefItem(data.item, data.name)}`;
      }
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

const displayType = (type: string): string => {
  if (type.endsWith('[]')) {
    return `Array of ${displayType(type.substring(0, type.length - 2))}`;
  }

  if (/^(u|i)(size|\d{1,2})$/gm.test(type)) {
    return 'Number';
  } else if (type == 'bool') {
    return 'Boolean';
  } else if (type == 'str') {
    return 'String';
  } else if (type == 'file') {
    return 'File';
  }

  let entry = AUTODOC_ENTRIES.items.find((entry) => entry.endsWith(`/${type}.json`))?.split('.')[0];
  return entry ? `[${type}](/reference/${entry})` : type;
};

const uncodeName = (name: string): string => {
  return (
    name
      // snake_case
      .replace(
        /([a-zA-Z]+)_([a-zA-Z]+)/gm,
        (_, p1: string, p2: string) =>
          `${p1[0].toUpperCase()}${p1.slice(1).toLowerCase()}${p2[0].toUpperCase()}${p2
            .slice(1)
            .toLowerCase()}`
      )
      // UPPER -> lower
      .replace(/^[A-Z]+$/gm, (p1: string) => p1.toLowerCase())
      // _underscore
      .replace(/(?:^|_)([a-z0-9])/gm, (_, p1: string) => p1.toUpperCase())
      // Title Case
      .replace(/[A-Z]/gm, ' $&')
      // Remove any remaining underscores
      .replace(/_/gm, '')
      .trim()
  );
};

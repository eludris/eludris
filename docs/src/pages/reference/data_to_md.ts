import { readFileSync } from 'fs';
import { FieldInfo, ItemInfo, ItemType, StructInfo } from "../../lib/types";
import AUTODOC_ENTRIES from '../../../public/autodoc/index.json';

export default (data: ItemInfo, name: string): string => {
  let content = `# ${name}`;
  let example: string | null = null;
  if (data.type == ItemType.Route) {
    content += `\n\n<span class="method">${data.method}</span><span class="route">${data.route.replace(/<.+?>/gm, '<span class="special-segment">$&</span>')}</span>`;
  }
  if (data.doc) {
    const parts = data.doc.split('-----');
    let doc = parts.shift();
    example = parts.join('-----');
    content += `\n\n${doc}`
  }
  if (data.type == ItemType.Struct) {
    content += '\n\n|Field|Type|Description|\n|---|---|---|';
    data.fields.forEach((field) => {
      content += `\n${display_field(field)}`
    });
  }
  if (example) {
    content += `\n${example}`;
  }
  return content;
};

const display_field = (field: FieldInfo): string => {
  const innerType = field.flattened && AUTODOC_ENTRIES.find((entry) => entry.endsWith(`/${field.name}`));
  if (innerType) {
    let innerData: StructInfo = JSON.parse(readFileSync(`autodoc/${innerType}`).toString());
    let fields = '';
    innerData.fields.forEach((field) => {
      fields += `\n${display_field(field)}`;
    })
    return fields;
  }
  return `|${field.name}${field.ommitable ? '?' : ''}|${display_type(field.field_type)}${field.nullable ? '?' : ''}|${field.doc?.replace(/(\S)\n(\S)/gm, '$1 $2') ?? ''}|`;
}

const display_type = (type: string): string => {
  if (type == 'u32' || type == 'u64' || type == 'usize') {
    return 'Number';
  }
  return type.replace(/Option<(.+)>/gm, '$1').replace(/</gm, '\\<');
}

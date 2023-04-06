import { z, defineCollection } from 'astro:content';

export const collections = {
  docs: defineCollection({
    schema: z.object({
      title: z.string(),
      description: z.string(),
      order: z.number()
    })
  }),
  changelogs: defineCollection({
    schema: z.object({
      version: z.string(),
      date: z.string(),
      pr: z.number()
    })
  }),
  extra: defineCollection({
    schema: z.object({
      title: z.string(),
      description: z.string(),
      order: z.number()
    })
  })
};

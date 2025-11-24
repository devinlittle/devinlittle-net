/* import { fetchMarkdownPosts } from "$lib/utils";
import { json } from "@sveltejs/kit";

export const GET = async () => {
  const allPosts = await fetchMarkdownPosts();

  const sortedPosts = allPosts.sort((v, vv) => {
    return new Date(vv.meta.date) - new Date(v.meta.date);
  });

  return json(sortedPosts);
}; */

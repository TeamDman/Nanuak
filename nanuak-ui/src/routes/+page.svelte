<script lang="ts">
  import { onMount } from 'svelte';
  import SearchBox from '$components/SearchBox.svelte';
  import ThumbnailGrid from '$components/ThumbnailGrid.svelte';
  import { commands } from '$lib/bindings';
  import type { Video } from '$lib/bindings';

  let videos: Video[] = [];
  let filteredVideos: Video[] = [];

  const fetchData = async (query: string = '') => {
    const data = await commands.fetchVideos(query);
    if (data.status === 'ok') {
      videos = data.data;
      filteredVideos = data.data;
    } else {
      console.error(data.error);
    }
  };

  const handleSearch = async (query: string) => {
    await fetchData(query);
  };

  onMount(() => fetchData());
</script>

<div class="min-h-screen bg-gray-50 py-8">
  <div class="container mx-auto space-y-6">
    <h1 class="text-3xl font-bold">YouTube Watch History</h1>
    <SearchBox onSearch={handleSearch} />
    <ThumbnailGrid videos={filteredVideos} />
  </div>
</div>

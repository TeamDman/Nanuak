<script lang="ts">
  import { onMount } from 'svelte';
  import SearchBox from '$components/SearchBox.svelte';
  import ThumbnailGrid from '$components/ThumbnailGrid.svelte';
  import { commands } from '$lib/bindings'; // Custom API for data fetching
  import type { Video } from '$lib/bindings'; // Custom type for video data
  let videos: Video[] = [];
  let filteredVideos: Video[] = [];

  const fetchData = async () => {
    // Replace with your Tauri backend call
    // const data = await invoke('fetch_videos');
    const data = await commands.fetchVideos();
    videos = data;
    filteredVideos = data;
  };

  const handleSearch = (query: string) => {
    filteredVideos = videos.filter((video) =>
      video.title.toLowerCase().includes(query.toLowerCase())
    );
  };

  onMount(fetchData);
</script>

<div class="min-h-screen bg-gray-50 py-8">
  <div class="container mx-auto space-y-6">
    <h1 class="text-3xl font-bold">YouTube Watch History</h1>
    <SearchBox onSearch={handleSearch} />
    <ThumbnailGrid videos={filteredVideos} />
  </div>
</div>

<script lang="ts">
  import { onMount } from 'svelte';
  import { commands } from '$lib/bindings';
  import type { Video } from '$lib/bindings';

  let query = '';
  let videos: Video[] = [];

  const fetchData = async (search: string = '') => {
    const data = await commands.fetchVideos(search);
    if (data.status === 'ok') {
      videos = data.data;
    } else {
      console.error(data.error);
    }
  };

  const handleSearch = async () => {
    await fetchData(query);
  };

  onMount(() => fetchData());
</script>

<!-- 
    Notes:
    - Dark mode with sleek glassy blurs: use dark background, semi-transparent panels, backdrop filters.
    - Rainbow gradients: background or subtle overlay.
    - Fancy background grids: maybe a subtle SVG pattern or gradient lines.
    - Tailwind classes to create a modern, minimal, yet colorful UI.
    
    French learning aside:
    "How do I search?" -> "Comment puis-je faire une recherche ?"
  -->

<!-- Outer container with a fancy gradient background and subtle grid pattern -->
<div
  class="relative min-h-screen flex flex-col bg-gradient-to-b from-gray-900 via-black to-gray-900 text-gray-100"
>
  <!-- Background decorative elements -->
  <div class="absolute inset-0 pointer-events-none overflow-hidden">
    <!-- A subtle grid pattern or gradient lines -->
    <div
      class="absolute inset-0 bg-grid-pattern opacity-5 [mask-image:radial-gradient(white,transparent,transparent)]"
    ></div>
    <!-- Some floating rainbow gradient blur -->
    <div
      class="absolute -top-1/4 left-1/2 w-[60rem] h-[60rem] -translate-x-1/2 bg-[conic-gradient(at_top,_var(--tw-gradient-stops))] from-pink-500 via-purple-500 to-indigo-500 blur-3xl opacity-30"
    ></div>
  </div>

  <!-- Header -->
  <header class="relative z-10 p-4 flex items-center justify-between">
    <h1 class="text-3xl font-extrabold tracking-tight">
      <span
        class="bg-clip-text text-transparent bg-gradient-to-r from-indigo-400 via-pink-500 to-red-400"
      >
        Your YouTube Watch History
      </span>
    </h1>
    <div class="flex space-x-2 items-center">
      <input
        type="text"
        placeholder="Search..."
        bind:value={query}
        on:keyup={(e) => e.key === 'Enter' && handleSearch()}
        class="rounded-full bg-white/5 backdrop-blur-sm px-4 py-2 text-gray-100 focus:outline-none focus:ring-2 focus:ring-pink-500 placeholder:text-gray-400 w-64"
      />
      <button
        on:click={handleSearch}
        class="rounded-full px-4 py-2 bg-gradient-to-tr from-pink-500 to-indigo-500 hover:scale-105 transition-transform font-medium"
      >
        Search
      </button>
    </div>
  </header>

  <!-- Content -->
  <main class="relative z-10 flex-grow p-4 md:p-8">
    {#if videos.length === 0}
      <div class="flex flex-col items-center justify-center h-full text-center space-y-4">
        <p class="text-xl text-gray-300">No videos found. Try searching for something else.</p>
        <!-- French learning moment -->
        <p class="text-sm text-gray-500 italic">
          Tip: How do I search? <span class="text-gray-400"
            >"Comment puis-je faire une recherche ?"</span
          >
        </p>
      </div>
    {:else}
      <div class="grid gap-6 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4">
        {#each videos as video}
          <div
            class="group flex flex-col items-center rounded-lg bg-white/5 backdrop-blur-sm p-4 hover:bg-white/10 transition-colors overflow-hidden"
          >
            <div class="relative w-full aspect-video rounded-lg overflow-hidden">
              <img
                src={video.thumbnail}
                alt={video.title}
                class="w-full h-full object-cover transition-transform group-hover:scale-105"
              />
              <div
                class="absolute bottom-2 right-2 bg-black/70 text-white text-xs rounded px-1 py-0.5"
              >
                {Math.floor(video.duration / 60)}:{String(video.duration % 60).padStart(2, '0')}
              </div>
            </div>
            <h2 class="mt-4 text-lg font-semibold line-clamp-2 text-center">{video.title}</h2>
            <p class="mt-1 text-sm text-gray-400">{video.views} views</p>
          </div>
        {/each}
      </div>
    {/if}
  </main>
</div>

<style>
  /* Subtle grid pattern (use your own pattern if desired) */
  .bg-grid-pattern {
    background-image: linear-gradient(to right, rgba(255, 255, 255, 0.05) 1px, transparent 1px),
      linear-gradient(to bottom, rgba(255, 255, 255, 0.05) 1px, transparent 1px);
    background-size: 20px 20px;
  }
</style>

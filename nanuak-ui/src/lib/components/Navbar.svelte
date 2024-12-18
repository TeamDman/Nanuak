<script lang="ts">
  import { page } from '$app/stores';
  import { cn } from '$lib/utils';
  let className: string | undefined | null = undefined;
  export { className as class };

  const links = [
    {
      name: 'Home',
      href: '/'
    },
    {
      name: 'Bruh',
      href: '/bruh'
    }
  ];
</script>

N
<div class="w-full">
  <div class={cn('', className)} {...$$restProps}>
    {#each links as link, index (index)}
      <a
        href={link.href}
        class={cn(
          'px-2',
          $page.url.pathname.startsWith(link.href)
            ? 'font-bold text-primary'
            : 'font-medium text-muted-foreground'
        )}
      >
        {link.name}
      </a>
    {/each}
  </div>
</div>

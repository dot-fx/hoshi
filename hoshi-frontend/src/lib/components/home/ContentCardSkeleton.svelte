<script lang="ts">
    import { AspectRatio } from '$lib/components/ui/aspect-ratio';
    import { Skeleton } from "$lib/components/ui/skeleton";
</script>

<div class="block w-full outline-none">
    <div class="relative overflow-hidden rounded-xl">
        <AspectRatio ratio={2/3}>
            <div class="h-full w-full">
                <Skeleton class="h-full w-full" />
            </div>
        </AspectRatio>
    </div>

    <div class="mt-3 space-y-1">
        <div class="min-h-10 space-y-1">
            <Skeleton class="h-3.5 w-full" />
            <Skeleton class="h-3.5 w-4/5" />
        </div>

        <div class="flex items-center gap-2 text-xs h-[16px]">
            <Skeleton class="h-3 w-8" />
            <Skeleton class="h-3 w-16" />
        </div>
    </div>
</div>
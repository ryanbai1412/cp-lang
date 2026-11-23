'use client';
import { useDocsSearch } from 'fumadocs-core/search/client';
import { staticClient } from 'fumadocs-core/search/client/orama-static';
import {
  SearchDialog,
  SearchDialogClose,
  SearchDialogContent,
  SearchDialogHeader,
  SearchDialogIcon,
  SearchDialogInput,
  SearchDialogList,
  SearchDialogListItem,
  SearchDialogOverlay,
  type SearchItemType,
} from 'fumadocs-ui/components/dialog/search';
import type { SharedProps } from 'fumadocs-ui/contexts/search';
import type { MouseEvent } from 'react';

// Fumadocs' default dialog, except that Ctrl/Cmd-click or a middle click on a
// result opens it in a new tab, the way a link would.
function Item({ item, onClick }: { item: SearchItemType; onClick: () => void }) {
  const url = 'url' in item ? item.url : undefined;
  const newTab = (e: MouseEvent) => {
    if (!url) return false;
    e.preventDefault();
    window.open(url, '_blank', 'noopener');
    return true;
  };
  return (
    <SearchDialogListItem
      item={item}
      onClick={(e: MouseEvent) => ((e.metaKey || e.ctrlKey) && newTab(e)) || onClick()}
      onAuxClick={(e: MouseEvent) => e.button === 1 && newTab(e)}
    />
  );
}

export default function Search(props: SharedProps) {
  const search = useDocsSearch({ client: staticClient({}) });
  return (
    <SearchDialog {...search} {...props}>
      <SearchDialogOverlay />
      <SearchDialogContent>
        <SearchDialogHeader>
          <SearchDialogIcon />
          <SearchDialogInput />
          <SearchDialogClose />
        </SearchDialogHeader>
        <SearchDialogList Item={Item} />
      </SearchDialogContent>
    </SearchDialog>
  );
}

package rs.compio.winio;

import android.view.View;
import android.view.ViewGroup;
import android.widget.FrameLayout;

import androidx.recyclerview.widget.RecyclerView;

import java.util.ArrayList;
import java.util.List;

public class TabViewAdapter extends RecyclerView.Adapter<TabViewAdapter.ViewHolder> {
    private List<View> pages;

    public TabViewAdapter() {
        this.pages = new ArrayList<>();
    }

    public List<View> getPages() {
        return pages;
    }

    @Override
    public ViewHolder onCreateViewHolder(ViewGroup parent, int viewType) {
        FrameLayout item = new FrameLayout(parent.getContext());
        item.setLayoutParams(new ViewGroup.LayoutParams(
                FrameLayout.LayoutParams.MATCH_PARENT,
                FrameLayout.LayoutParams.MATCH_PARENT
        ));
        return new ViewHolder(item);
    }

    @Override
    public void onBindViewHolder(ViewHolder holder, int position) {
        FrameLayout itemView = (FrameLayout) holder.itemView;
        itemView.removeAllViews();
        View page = pages.get(position);
        ViewGroup parent = (ViewGroup) page.getParent();
        if (parent != null) {
            parent.removeView(page);
        }
        itemView.addView(page);
        page.setLayoutParams(new FrameLayout.LayoutParams(
                FrameLayout.LayoutParams.MATCH_PARENT,
                FrameLayout.LayoutParams.MATCH_PARENT
        ));
    }

    @Override
    public int getItemCount() {
        return pages.size();
    }

    static class ViewHolder extends RecyclerView.ViewHolder {
        ViewHolder(FrameLayout itemView) {
            super(itemView);
        }
    }
}

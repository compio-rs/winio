package rs.compio.winio;

import android.view.View;
import android.widgets.FrameLayout;
import androidx.recyclerview.widget.RecyclerView;
import androidx.viewpager2.widget.ViewPager2;

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
    public TabViewHolder onCreateViewHolder(ViewGroup parent, int viewType) {
        return new TabViewHolder(new FrameLayout(parent.getContext()));
    }

    @Override
    public void onBindViewHolder(TabViewHolder holder, int position) {
        holder.itemView.removeAllViews();
        View page = pages.get(position);
        holder.itemView.addView(page);
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

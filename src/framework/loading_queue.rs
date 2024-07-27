use bevy::prelude::*;

#[derive(Resource)]
pub struct AssetLoadingQueue<T: Asset> {
    loading: Vec<Handle<T>>,
}
impl<T: Asset> Default for AssetLoadingQueue<T> {
    fn default() -> Self {
        Self { loading: vec![] }
    }
}

#[derive(Event)]
pub struct AssetLoadingCompleted<T: Asset> {
    pub handle: Handle<T>,
}

#[derive(Event)]
pub struct WatchAssetLoading<T: Asset> {
    handle: Handle<T>,
}

impl<T: Asset> WatchAssetLoading<T> {
    pub fn new(handle: Handle<T>) -> Self {
        Self { handle }
    }
}

pub fn add_watches<T>(
    mut queue: ResMut<AssetLoadingQueue<T>>,
    mut watch: EventReader<WatchAssetLoading<T>>,
) where
    T: Asset,
{
    for watch in watch.read() {
        if !queue.loading.contains(&watch.handle) {
            queue.loading.push(watch.handle.clone());
        }
    }
}

pub fn process_asset_loading_queue<T>(
    mut queue: ResMut<AssetLoadingQueue<T>>,
    mut complete: EventWriter<AssetLoadingCompleted<T>>,
    assets: Res<Assets<T>>,
) where
    T: Asset,
{
    let mut requeue = vec![];
    for handle in queue.loading.drain(..) {
        if assets.get(&handle).is_none() {
            requeue.push(handle);
        } else {
            complete.send(AssetLoadingCompleted { handle });
        }
    }
    queue.loading = requeue;
}

pub fn initialize<T: Asset>(app: &mut App) {
    app.init_resource::<AssetLoadingQueue<T>>()
        .add_event::<WatchAssetLoading<T>>()
        .add_event::<AssetLoadingCompleted<T>>()
        .add_systems(Update, add_watches::<T>)
        .add_systems(
            Update,
            process_asset_loading_queue::<T>.after(add_watches::<T>),
        );
}

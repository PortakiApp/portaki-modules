//! Statistics — the helpers behind the tiles (`statsSummary`) and details for « Checklist »
//! (guest lists) and « Ménage » (host cleaning lists).
//!
//! Only what the module stores is shown: guest ticks per stay, host ticks per task. The platform
//! passes the period's stays to the detail page (`input.stays` — dates and status, no guest name):
//! with them, a stay that never opened its list shows as « Non remplie » and a cleaning is judged
//! against its deadline. Without them (developer preview, older platform), both stay unknown.

use std::collections::BTreeMap;

use chrono::{DateTime, Duration, Utc};
use portaki_sdk::contracts::timeline::TimelineStay;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, Chart, EmptyState, FeedItem, Grid, Stack, Stat};
use portaki_sdk::sdui::{ChartKind, ChartPoint, FeedStatus};
use uuid::Uuid;

use crate::commands::task::{parse_task_id, plan_tasks, task_id};
use crate::entities::TaskItemState;
use crate::storage;
use crate::{lists, List, Step};

mod cleaning_rows;
mod cleaning_tasks;
mod guest;
mod period;
mod shared;

pub(crate) use cleaning_rows::*;
pub(crate) use cleaning_tasks::*;
pub(crate) use guest::*;
pub(crate) use period::*;
pub(crate) use shared::*;

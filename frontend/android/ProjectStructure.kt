// ─── Android Project Structure (Kotlin + Jetpack Compose) ───
//
// com.cardeal.app/
// ├── MainActivity.kt
// ├── CarDealApp.kt                ← NavHost + theme
// ├── di/
// │   └── AppModule.kt             ← Hilt dependency injection
// ├── data/
// │   ├── api/
// │   │   ├── CarDealApi.kt        ← Retrofit interface
// │   │   └── ApiClient.kt         ← OkHttp + interceptors
// │   ├── models/
// │   │   └── VehicleReport.kt     ← (already created)
// │   └── repository/
// │       └── VehicleRepository.kt ← Data layer
// ├── ui/
// │   ├── theme/
// │   │   ├── Theme.kt
// │   │   ├── Color.kt
// │   │   └── Type.kt
// │   ├── screens/
// │   │   ├── splash/
// │   │   │   └── SplashScreen.kt
// │   │   ├── home/
// │   │   │   ├── HomeScreen.kt
// │   │   │   └── HomeViewModel.kt
// │   │   ├── report/
// │   │   │   ├── ReportScreen.kt
// │   │   │   ├── ReportViewModel.kt
// │   │   │   └── sections/
// │   │   │       ├── VehicleOverviewCard.kt
// │   │   │       ├── SpecsCard.kt
// │   │   │       ├── OwnershipCard.kt
// │   │   │       ├── InspectionCard.kt
// │   │   │       ├── RecallsCard.kt
// │   │   │       ├── TaxCostsCard.kt
// │   │   │       ├── PriceEstimateCard.kt
// │   │   │       ├── RiskCard.kt
// │   │   │       ├── ConditionCard.kt
// │   │   │       ├── SuitabilityCard.kt
// │   │   │       └── AiSummaryCard.kt
// │   │   ├── plans/
// │   │   │   ├── PlansScreen.kt
// │   │   │   └── PlansViewModel.kt
// │   │   └── profile/
// │   │       ├── ProfileScreen.kt
// │   │       └── ProfileViewModel.kt
// │   ├── components/
// │   │   ├── SearchBar.kt
// │   │   ├── ReportCard.kt
// │   │   ├── ScoreGauge.kt
// │   │   ├── PriceRangeBar.kt
// │   │   ├── TimelineView.kt
// │   │   ├── AlertBadge.kt
// │   │   ├── CreditBadge.kt
// │   │   └── LoadingOverlay.kt
// │   └── navigation/
// │       └── NavGraph.kt
// └── util/
//     ├── LocaleManager.kt
//     └── DateFormatter.kt
